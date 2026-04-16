// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Stack-based virtual machine that interprets UniLang bytecode.

use std::collections::HashMap;

use unilang_codegen::bytecode::{Bytecode, ClassDef, Function, Opcode};

use crate::error::{ErrorKind, RuntimeError};
use crate::value::{InstanceData, RuntimeValue};

/// A single activation record on the call stack.
struct CallFrame {
    /// Instructions for this frame.
    code: Vec<Opcode>,
    /// Instruction pointer (index of next instruction to execute).
    ip: usize,
    /// Local variable slots.
    locals: Vec<RuntimeValue>,
    /// Where this frame's operand stack starts.
    #[allow(dead_code)]
    stack_base: usize,
}

/// Type alias for native builtin function implementations.
pub type BuiltinFn = Box<dyn Fn(&[RuntimeValue]) -> Result<RuntimeValue, RuntimeError>>;

/// Maximum operand stack depth before a stack-overflow error is raised.
const MAX_STACK_DEPTH: usize = 100_000;
/// Maximum call-frame depth (recursion limit).
const MAX_CALL_DEPTH: usize = 10_000;

/// An active exception handler registered by `PushExceptHandler`.
struct ExceptHandler {
    /// Index of the `frames` entry that pushed this handler.
    frame_idx: usize,
    /// Instruction pointer to jump to when an exception is caught.
    catch_ip: usize,
    /// Operand-stack depth at the time the handler was pushed (used to restore
    /// the stack before entering the catch block).
    stack_depth: usize,
}

/// The UniLang virtual machine.
pub struct VM {
    /// Operand stack.
    stack: Vec<RuntimeValue>,
    /// Global variables.
    globals: HashMap<String, RuntimeValue>,
    /// Call stack.
    frames: Vec<CallFrame>,
    /// Function table (from bytecode).
    functions: Vec<Function>,
    /// Class definitions (from bytecode).
    classes: Vec<ClassDef>,
    /// Captured print output — None in normal execution, Some in test/capture mode.
    /// Never populated during normal runs so memory is not consumed by print calls.
    output: Option<Vec<String>>,
    /// Registered builtin functions (includes drivers registered via DriverRegistry).
    builtins: HashMap<String, BuiltinFn>,
    /// Active exception handlers (innermost first at the end).
    except_handlers: Vec<ExceptHandler>,
}

impl VM {
    /// Create a new VM (normal execution — output is NOT captured).
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            output: None,
            builtins: HashMap::new(),
            except_handlers: Vec::new(),
        }
    }

    /// Create a VM that captures print output (used by tests and `execute_with_output`).
    pub fn new_with_capture() -> Self {
        Self {
            output: Some(Vec::new()),
            ..Self::new()
        }
    }

    /// Return captured print output (empty slice if capture was not enabled).
    pub fn output(&self) -> &[String] {
        self.output.as_deref().unwrap_or(&[])
    }

    /// Register a native built-in function.
    pub fn register_builtin(
        &mut self,
        name: impl Into<String>,
        func: impl Fn(&[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> + 'static,
    ) {
        let name = name.into();
        self.globals
            .insert(name.clone(), RuntimeValue::NativeFunction(name.clone()));
        self.builtins.insert(name, Box::new(func));
    }

    /// Set a global variable value.
    pub fn set_global(&mut self, name: impl Into<String>, value: RuntimeValue) {
        self.globals.insert(name.into(), value);
    }

    /// Run bytecode to completion and return the last value (or Null).
    pub fn run(&mut self, bytecode: &Bytecode) -> Result<RuntimeValue, RuntimeError> {
        self.functions = bytecode.functions.clone();
        self.classes = bytecode.classes.clone();

        // Create the top-level frame.
        let frame = CallFrame {
            code: bytecode.instructions.clone(),
            ip: 0,
            locals: vec![RuntimeValue::Null; 256], // generous local slots
            stack_base: 0,
        };
        self.frames.push(frame);

        loop {
            let result = self.step();
            match result {
                Ok(true) => continue,
                Ok(false) => {
                    // Normal completion.
                    return Ok(self.stack.pop().unwrap_or(RuntimeValue::Null));
                }
                Err(e) if e.kind == ErrorKind::Halt => {
                    return Ok(self.stack.pop().unwrap_or(RuntimeValue::Null));
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ── Helpers ──────────────────────────────────────────

    fn pop(&mut self) -> Result<RuntimeValue, RuntimeError> {
        self.stack.pop().ok_or_else(RuntimeError::stack_underflow)
    }

    fn peek(&self) -> Result<&RuntimeValue, RuntimeError> {
        self.stack.last().ok_or_else(RuntimeError::stack_underflow)
    }

    fn push(&mut self, val: RuntimeValue) {
        if self.stack.len() >= MAX_STACK_DEPTH {
            // Abort with a clear message rather than silently consuming all RAM.
            eprintln!("runtime error: operand stack overflow (exceeded {} entries)", MAX_STACK_DEPTH);
            std::process::exit(1);
        }
        self.stack.push(val);
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("no active call frame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no active call frame")
    }

    // ── Execute one instruction, return Ok(true) to continue,
    //    Ok(false) when there are no more instructions. ─────────

    fn step(&mut self) -> Result<bool, RuntimeError> {
        if self.frames.is_empty() {
            return Ok(false);
        }

        let frame = self.frames.last().unwrap();
        if frame.ip >= frame.code.len() {
            // Frame exhausted without explicit Halt/Return.
            if self.frames.len() == 1 {
                return Ok(false);
            }
            // Pop the frame (implicit return Null).
            self.frames.pop();
            self.push(RuntimeValue::Null);
            return Ok(true);
        }

        // Fetch instruction.
        let instr = self.frames.last().unwrap().code[self.frames.last().unwrap().ip].clone();
        self.current_frame_mut().ip += 1;

        match self.execute(instr) {
            Ok(cont) => Ok(cont),
            Err(e) if e.kind == ErrorKind::Halt => Err(e),
            Err(e) => {
                // Check for an active exception handler in the current or any enclosing frame.
                let current_frame_idx = self.frames.len().saturating_sub(1);
                if let Some(handler_pos) = self
                    .except_handlers
                    .iter()
                    .rposition(|h| h.frame_idx == current_frame_idx)
                {
                    let handler = self.except_handlers.remove(handler_pos);
                    // Remove any nested handlers that were inside the try body.
                    self.except_handlers
                        .retain(|h| h.frame_idx < current_frame_idx);
                    // Restore the operand stack to the depth at the time the handler was registered.
                    self.stack.truncate(handler.stack_depth);
                    // Push the exception message so the catch block can bind it.
                    self.push(RuntimeValue::String(e.message.clone()));
                    // Jump to the catch block.
                    self.current_frame_mut().ip = handler.catch_ip;
                    Ok(true)
                } else {
                    Err(e)
                }
            }
        }
    }

    fn execute(&mut self, instr: Opcode) -> Result<bool, RuntimeError> {
        match instr {
            // ── Stack manipulation ───────────────────────────
            Opcode::LoadConst(v) => {
                let rv: RuntimeValue = v.into();
                self.push(rv);
            }

            Opcode::LoadLocal(slot) => {
                let val = self.current_frame().locals[slot].clone();
                self.push(val);
            }

            Opcode::StoreLocal(slot) => {
                let val = self.pop()?;
                self.current_frame_mut().locals[slot] = val;
            }

            Opcode::LoadGlobal(ref name) => {
                // Check if it's a built-in name.
                if name == "print" {
                    // Push a sentinel; print is handled specially via Call.
                    self.push(RuntimeValue::String("__builtin_print__".to_string()));
                } else if let Some(val) = self.globals.get(name) {
                    self.push(val.clone());
                } else {
                    return Err(RuntimeError::undefined_variable(name));
                }
            }

            Opcode::StoreGlobal(ref name) => {
                let val = self.pop()?;
                self.globals.insert(name.clone(), val);
            }

            Opcode::Pop => {
                let _ = self.pop()?;
            }

            Opcode::Dup => {
                let val = self.peek()?.clone();
                self.push(val);
            }

            // ── Arithmetic ───────────────────────────────────
            Opcode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (RuntimeValue::Int(x), RuntimeValue::Int(y)) => RuntimeValue::Int(x + y),
                    (RuntimeValue::Float(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x + y),
                    (RuntimeValue::Int(x), RuntimeValue::Float(y)) => {
                        RuntimeValue::Float(*x as f64 + y)
                    }
                    (RuntimeValue::Float(x), RuntimeValue::Int(y)) => {
                        RuntimeValue::Float(x + *y as f64)
                    }
                    // String + anything → coerce to string and concat
                    (RuntimeValue::String(_), _) | (_, RuntimeValue::String(_)) => {
                        RuntimeValue::String(format!(
                            "{}{}",
                            a.coerce_to_string(),
                            b.coerce_to_string()
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::type_error(format!(
                            "cannot add {} and {}",
                            a, b
                        )))
                    }
                };
                self.push(result);
            }

            Opcode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.numeric_op(&a, &b, |x, y| x - y, |x, y| x - y, "subtract")?;
                self.push(result);
            }

            Opcode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.numeric_op(&a, &b, |x, y| x * y, |x, y| x * y, "multiply")?;
                self.push(result);
            }

            Opcode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                // Division always produces float, but check for zero.
                let bf = b
                    .as_float()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot divide by {}", b)))?;
                let af = a
                    .as_float()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot divide {}", a)))?;
                if bf == 0.0 {
                    return Err(RuntimeError::division_by_zero());
                }
                self.push(RuntimeValue::Float(af / bf));
            }

            Opcode::FloorDiv => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (RuntimeValue::Int(x), RuntimeValue::Int(y)) => {
                        if *y == 0 {
                            return Err(RuntimeError::division_by_zero());
                        }
                        self.push(RuntimeValue::Int(x.div_euclid(*y)));
                    }
                    _ => {
                        let af = a.as_float().ok_or_else(|| {
                            RuntimeError::type_error(format!("cannot floor-divide {}", a))
                        })?;
                        let bf = b.as_float().ok_or_else(|| {
                            RuntimeError::type_error(format!("cannot floor-divide by {}", b))
                        })?;
                        if bf == 0.0 {
                            return Err(RuntimeError::division_by_zero());
                        }
                        self.push(RuntimeValue::Float((af / bf).floor()));
                    }
                }
            }

            Opcode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (RuntimeValue::Int(x), RuntimeValue::Int(y)) => {
                        if *y == 0 {
                            return Err(RuntimeError::division_by_zero());
                        }
                        self.push(RuntimeValue::Int(x.rem_euclid(*y)));
                    }
                    _ => {
                        let af = a
                            .as_float()
                            .ok_or_else(|| RuntimeError::type_error(format!("cannot mod {}", a)))?;
                        let bf = b.as_float().ok_or_else(|| {
                            RuntimeError::type_error(format!("cannot mod by {}", b))
                        })?;
                        if bf == 0.0 {
                            return Err(RuntimeError::division_by_zero());
                        }
                        self.push(RuntimeValue::Float(af % bf));
                    }
                }
            }

            Opcode::Pow => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (RuntimeValue::Int(x), RuntimeValue::Int(y)) => {
                        if *y >= 0 {
                            self.push(RuntimeValue::Int(x.pow(*y as u32)));
                        } else {
                            self.push(RuntimeValue::Float((*x as f64).powf(*y as f64)));
                        }
                    }
                    _ => {
                        let af = a.as_float().ok_or_else(|| {
                            RuntimeError::type_error(format!("cannot raise {} to power", a))
                        })?;
                        let bf = b.as_float().ok_or_else(|| {
                            RuntimeError::type_error(format!("cannot use {} as exponent", b))
                        })?;
                        self.push(RuntimeValue::Float(af.powf(bf)));
                    }
                }
            }

            Opcode::Neg => {
                let a = self.pop()?;
                let result = match a {
                    RuntimeValue::Int(n) => RuntimeValue::Int(-n),
                    RuntimeValue::Float(f) => RuntimeValue::Float(-f),
                    _ => return Err(RuntimeError::type_error(format!("cannot negate {}", a))),
                };
                self.push(result);
            }

            // ── Comparison ───────────────────────────────────
            Opcode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(RuntimeValue::Bool(a == b));
            }

            Opcode::NotEq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(RuntimeValue::Bool(a != b));
            }

            Opcode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.partial_cmp(&b).is_some_and(|o| o.is_lt());
                self.push(RuntimeValue::Bool(result));
            }

            Opcode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.partial_cmp(&b).is_some_and(|o| o.is_gt());
                self.push(RuntimeValue::Bool(result));
            }

            Opcode::LtEq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.partial_cmp(&b).is_some_and(|o| o.is_le());
                self.push(RuntimeValue::Bool(result));
            }

            Opcode::GtEq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.partial_cmp(&b).is_some_and(|o| o.is_ge());
                self.push(RuntimeValue::Bool(result));
            }

            // ── Logical ──────────────────────────────────────
            Opcode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(RuntimeValue::Bool(a.is_truthy() && b.is_truthy()));
            }

            Opcode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(RuntimeValue::Bool(a.is_truthy() || b.is_truthy()));
            }

            Opcode::Not => {
                let a = self.pop()?;
                self.push(RuntimeValue::Bool(!a.is_truthy()));
            }

            // ── Bitwise ──────────────────────────────────────
            Opcode::BitAnd => {
                let b = self.pop()?;
                let a = self.pop()?;
                let ai = a
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-AND {}", a)))?;
                let bi = b
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-AND {}", b)))?;
                self.push(RuntimeValue::Int(ai & bi));
            }

            Opcode::BitOr => {
                let b = self.pop()?;
                let a = self.pop()?;
                let ai = a
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-OR {}", a)))?;
                let bi = b
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-OR {}", b)))?;
                self.push(RuntimeValue::Int(ai | bi));
            }

            Opcode::BitXor => {
                let b = self.pop()?;
                let a = self.pop()?;
                let ai = a
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-XOR {}", a)))?;
                let bi = b
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-XOR {}", b)))?;
                self.push(RuntimeValue::Int(ai ^ bi));
            }

            Opcode::BitNot => {
                let a = self.pop()?;
                let ai = a
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot bitwise-NOT {}", a)))?;
                self.push(RuntimeValue::Int(!ai));
            }

            Opcode::LShift => {
                let b = self.pop()?;
                let a = self.pop()?;
                let ai = a
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot left-shift {}", a)))?;
                let bi = b
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot shift by {}", b)))?;
                self.push(RuntimeValue::Int(ai << bi));
            }

            Opcode::RShift => {
                let b = self.pop()?;
                let a = self.pop()?;
                let ai = a
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot right-shift {}", a)))?;
                let bi = b
                    .as_int()
                    .ok_or_else(|| RuntimeError::type_error(format!("cannot shift by {}", b)))?;
                self.push(RuntimeValue::Int(ai >> bi));
            }

            // ── String ───────────────────────────────────────
            Opcode::Concat => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(RuntimeValue::String(format!("{}{}", a, b)));
            }

            // ── Control flow ─────────────────────────────────
            Opcode::Jump(target) => {
                self.current_frame_mut().ip = target;
            }

            Opcode::JumpIfFalse(target) => {
                let val = self.pop()?;
                if !val.is_truthy() {
                    self.current_frame_mut().ip = target;
                }
            }

            Opcode::JumpIfTrue(target) => {
                let val = self.pop()?;
                if val.is_truthy() {
                    self.current_frame_mut().ip = target;
                }
            }

            // ── Functions ────────────────────────────────────
            Opcode::Call(n_args) => {
                // The callee is on top of the stack (pushed after args).
                let callee = self.pop()?;

                match callee {
                    RuntimeValue::Function(idx) => {
                        let func = self.functions[idx].clone();
                        let stack_base = self.stack.len() - n_args;

                        // Build locals: first n_args slots are the arguments.
                        let mut locals = vec![RuntimeValue::Null; func.local_count.max(n_args)];
                        locals[..n_args]
                            .clone_from_slice(&self.stack[stack_base..stack_base + n_args]);
                        // Remove arguments from the stack.
                        self.stack.truncate(stack_base);

                        if self.frames.len() >= MAX_CALL_DEPTH {
                            return Err(RuntimeError::new(
                                ErrorKind::Exception,
                                format!("maximum call depth exceeded ({})", MAX_CALL_DEPTH),
                            ));
                        }
                        let frame = CallFrame {
                            code: func.code.clone(),
                            ip: 0,
                            locals,
                            stack_base,
                        };
                        self.frames.push(frame);
                    }
                    RuntimeValue::String(ref s) if s == "__builtin_print__" => {
                        // Built-in print: pop n_args and print them.
                        let mut args = Vec::new();
                        for _ in 0..n_args {
                            args.push(self.pop()?);
                        }
                        args.reverse();
                        let text = args
                            .iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!("{}", text);
                        if let Some(ref mut out) = self.output { out.push(text); }
                        self.push(RuntimeValue::Null);
                    }
                    RuntimeValue::NativeFunction(ref name) => {
                        let name = name.clone();
                        let mut args = Vec::new();
                        for _ in 0..n_args {
                            args.push(self.pop()?);
                        }
                        args.reverse();
                        let result = self.call_builtin(&name, &args)?;
                        self.push(result);
                    }
                    RuntimeValue::Class(ref class_name) => {
                        // Instantiate: create an empty instance, call __init__ if present.
                        let class_name = class_name.clone();
                        let mut args = Vec::with_capacity(n_args);
                        for _ in 0..n_args {
                            args.push(self.pop()?);
                        }
                        args.reverse();

                        // Build the initial instance.
                        let instance = RuntimeValue::Instance(InstanceData {
                            class_name: class_name.clone(),
                            fields: HashMap::new(),
                        });

                        // Look for __init__ in the class.
                        let init_idx = self
                            .classes
                            .iter()
                            .find(|c| c.name == class_name)
                            .and_then(|cd| {
                                cd.methods.iter().copied().find(|&idx| {
                                    self.functions
                                        .get(idx)
                                        .map_or(false, |f| f.name == "__init__")
                                })
                            });

                        if let Some(idx) = init_idx {
                            // Inject `this` as a global before calling __init__.
                            self.globals.insert("this".to_string(), instance.clone());
                            // Call __init__ with the provided args (NOT including `this`).
                            self.call_unilang_function(idx, args)?;
                            // Recover the (possibly mutated) `this` from globals.
                            let final_instance = self
                                .globals
                                .remove("this")
                                .unwrap_or(instance);
                            self.push(final_instance);
                        } else {
                            // No __init__ — push the empty instance directly.
                            self.push(instance);
                        }
                    }
                    _ => {
                        return Err(RuntimeError::type_error(format!(
                            "{} is not callable",
                            callee
                        )));
                    }
                }
            }

            Opcode::Return => {
                let retval = self.pop().unwrap_or(RuntimeValue::Null);
                // Pop the current frame.
                self.frames.pop();
                if self.frames.is_empty() {
                    // Returning from top-level; just push the value.
                    self.push(retval);
                    return Ok(false);
                }
                self.push(retval);
            }

            Opcode::MakeFunction(idx) => {
                self.push(RuntimeValue::Function(idx));
            }

            // ── Objects / classes ────────────────────────────
            Opcode::GetAttr(ref name) => {
                let obj = self.pop()?;
                match obj {
                    RuntimeValue::Instance(data) => {
                        let val = data
                            .fields
                            .get(name)
                            .cloned()
                            .ok_or_else(|| RuntimeError::attribute_error(name))?;
                        self.push(val);
                    }
                    _ => {
                        return Err(RuntimeError::type_error(format!(
                            "cannot get attribute '{}' of {}",
                            name, obj
                        )));
                    }
                }
            }

            Opcode::SetAttr(ref name) => {
                let value = self.pop()?;
                let obj = self.pop()?;
                match obj {
                    RuntimeValue::Instance(mut data) => {
                        data.fields.insert(name.clone(), value);
                        self.push(RuntimeValue::Instance(data));
                    }
                    _ => {
                        return Err(RuntimeError::type_error(format!(
                            "cannot set attribute '{}' on {}",
                            name, obj
                        )));
                    }
                }
            }

            Opcode::MakeClass(ref name, _member_count) => {
                // Class definitions are registered in self.classes at bytecode load time.
                // Push a Class descriptor so that calling ClassName(args) creates an instance.
                self.push(RuntimeValue::Class(name.clone()));
            }

            Opcode::NewInstance(ref name) => {
                let fields = HashMap::new();
                self.push(RuntimeValue::Instance(InstanceData {
                    class_name: name.clone(),
                    fields,
                }));
            }

            // ── Collections ──────────────────────────────────
            Opcode::MakeList(n) => {
                let start = self.stack.len().saturating_sub(n);
                let items: Vec<RuntimeValue> = self.stack.drain(start..).collect();
                self.push(RuntimeValue::List(items));
            }

            Opcode::MakeDict(n) => {
                let item_count = n * 2;
                let start = self.stack.len().saturating_sub(item_count);
                let items: Vec<RuntimeValue> = self.stack.drain(start..).collect();
                let mut pairs = Vec::with_capacity(n);
                for chunk in items.chunks_exact(2) {
                    pairs.push((chunk[0].clone(), chunk[1].clone()));
                }
                self.push(RuntimeValue::Dict(pairs));
            }

            Opcode::GetIndex => {
                let index = self.pop()?;
                let collection = self.pop()?;
                match (&collection, &index) {
                    (RuntimeValue::List(items), RuntimeValue::Int(i)) => {
                        let idx = if *i < 0 {
                            (items.len() as i64 + i) as usize
                        } else {
                            *i as usize
                        };
                        if idx >= items.len() {
                            return Err(RuntimeError::index_out_of_bounds(*i, items.len()));
                        }
                        self.push(items[idx].clone());
                    }
                    (RuntimeValue::Dict(pairs), _) => {
                        let val = pairs
                            .iter()
                            .find(|(k, _)| k == &index)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(RuntimeValue::Null);
                        self.push(val);
                    }
                    (RuntimeValue::String(s), RuntimeValue::Int(i)) => {
                        let idx = if *i < 0 {
                            (s.len() as i64 + i) as usize
                        } else {
                            *i as usize
                        };
                        if idx >= s.len() {
                            return Err(RuntimeError::index_out_of_bounds(*i, s.len()));
                        }
                        let ch = s.chars().nth(idx).unwrap();
                        self.push(RuntimeValue::String(ch.to_string()));
                    }
                    _ => {
                        return Err(RuntimeError::type_error(format!(
                            "cannot index {} with {}",
                            collection, index
                        )));
                    }
                }
            }

            Opcode::SetIndex => {
                let value = self.pop()?;
                let index = self.pop()?;
                let collection = self.pop()?;
                match (collection, &index) {
                    (RuntimeValue::List(mut items), RuntimeValue::Int(i)) => {
                        let idx = if *i < 0 {
                            (items.len() as i64 + i) as usize
                        } else {
                            *i as usize
                        };
                        if idx >= items.len() {
                            return Err(RuntimeError::index_out_of_bounds(*i, items.len()));
                        }
                        items[idx] = value;
                        self.push(RuntimeValue::List(items));
                    }
                    (RuntimeValue::Dict(mut pairs), _) => {
                        if let Some(pair) = pairs.iter_mut().find(|(k, _)| k == &index) {
                            pair.1 = value;
                        } else {
                            pairs.push((index, value));
                        }
                        self.push(RuntimeValue::Dict(pairs));
                    }
                    (other, _) => {
                        return Err(RuntimeError::type_error(format!(
                            "cannot set index on {}",
                            other
                        )));
                    }
                }
            }

            // ── Built-ins ────────────────────────────────────
            Opcode::Print => {
                let val = self.pop()?;
                let text = format!("{}", val);
                println!("{}", text);
                if let Some(ref mut out) = self.output { out.push(text); }
            }

            // ── Method calls ─────────────────────────────────
            Opcode::CallMethod(ref name, n_args) => {
                let name = name.clone();
                let mut args = Vec::with_capacity(n_args);
                for _ in 0..n_args {
                    args.push(self.pop()?);
                }
                args.reverse();
                let obj = self.pop()?;
                let result = self.dispatch_method(obj, &name, &args)?;
                self.push(result);
            }

            // ── Membership test ──────────────────────────────
            Opcode::Contains => {
                let container = self.pop()?;
                let item = self.pop()?;
                let found = match &container {
                    RuntimeValue::List(items) => items.contains(&item),
                    RuntimeValue::String(s) => match &item {
                        RuntimeValue::String(sub) => s.contains(sub.as_str()),
                        _ => false,
                    },
                    RuntimeValue::Dict(pairs) => pairs.iter().any(|(k, _)| k == &item),
                    _ => false,
                };
                self.push(RuntimeValue::Bool(found));
            }

            // ── Assertions & exceptions ──────────────────────
            Opcode::Assert => {
                let msg = self.pop()?;
                let cond = self.pop()?;
                if !cond.is_truthy() {
                    let msg_str = format!("{}", msg);
                    return Err(RuntimeError::new(
                        crate::error::ErrorKind::AssertionError,
                        msg_str,
                    ));
                }
            }

            Opcode::Raise => {
                let val = self.pop()?;
                return Err(RuntimeError::new(
                    crate::error::ErrorKind::Exception,
                    format!("{}", val),
                ));
            }

            // ── Exception handling ────────────────────────────
            Opcode::PushExceptHandler(catch_ip) => {
                self.except_handlers.push(ExceptHandler {
                    frame_idx: self.frames.len().saturating_sub(1),
                    catch_ip,
                    stack_depth: self.stack.len(),
                });
            }

            Opcode::PopExceptHandler => {
                self.except_handlers.pop();
            }

            Opcode::StoreExceptVar(ref name) => {
                // The exception message was pushed onto the stack by the error handler in step().
                let exc = self.pop()?;
                self.globals.insert(name.clone(), exc);
            }

            // ── Halt ─────────────────────────────────────────
            Opcode::Halt => {
                return Err(RuntimeError::halt());
            }
        }

        Ok(true)
    }

    /// Dispatch a method call — routes to String / List / Dict / Instance handlers.
    fn dispatch_method(
        &mut self,
        obj: RuntimeValue,
        name: &str,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue, RuntimeError> {
        match obj {
            RuntimeValue::String(s) => Self::string_method(s, name, args),
            RuntimeValue::List(items) => Self::list_method(items, name, args),
            RuntimeValue::Dict(pairs) => Self::dict_method(pairs, name, args),
            RuntimeValue::Instance(ref data) => {
                let class_name = data.class_name.clone();
                // Try field lookup first (e.g. System.out.println).
                if let Some(val) = data.fields.get(name).cloned() {
                    return match val {
                        RuntimeValue::NativeFunction(fn_name) => {
                            self.call_builtin(&fn_name, args)
                        }
                        other => Ok(other),
                    };
                }
                // Field not found — look up the method in the class table.
                let func_idx = self
                    .classes
                    .iter()
                    .find(|c| c.name == class_name)
                    .and_then(|class_def| {
                        class_def.methods.iter().copied().find(|&idx| {
                            self.functions
                                .get(idx)
                                .map_or(false, |f| f.name == name)
                        })
                    });
                if let Some(idx) = func_idx {
                    // Inject `this` into globals so the method body can access instance fields.
                    let prev_this = self.globals.remove("this");
                    self.globals.insert("this".to_string(), obj.clone());
                    let result = self.call_unilang_function(idx, args.to_vec());
                    // Restore previous `this` (or remove it if there was none).
                    if let Some(prev) = prev_this {
                        self.globals.insert("this".to_string(), prev);
                    } else {
                        self.globals.remove("this");
                    }
                    result
                } else {
                    Err(RuntimeError::attribute_error(name))
                }
            }
            _ => Err(RuntimeError::type_error(format!(
                "'{}' has no method '{}'",
                obj, name
            ))),
        }
    }

    // ── String method dispatch ────────────────────────────────

    fn string_method(s: String, name: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
        match name {
            "upper" | "toUpperCase" => Ok(RuntimeValue::String(s.to_uppercase())),
            "lower" | "toLowerCase" => Ok(RuntimeValue::String(s.to_lowercase())),
            "strip" | "trim" => Ok(RuntimeValue::String(s.trim().to_string())),
            "lstrip" | "trimLeft" | "trimStart" => Ok(RuntimeValue::String(s.trim_start().to_string())),
            "rstrip" | "trimRight" | "trimEnd" => Ok(RuntimeValue::String(s.trim_end().to_string())),
            "capitalize" => {
                let mut c = s.chars();
                let r = match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                };
                Ok(RuntimeValue::String(r))
            }
            "title" => {
                let r = s.split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(RuntimeValue::String(r))
            }
            "split" => {
                let sep = args.first().and_then(|a| a.as_string()).unwrap_or(" ").to_string();
                let parts: Vec<RuntimeValue> = if sep.is_empty() {
                    s.chars().map(|c| RuntimeValue::String(c.to_string())).collect()
                } else {
                    s.split(sep.as_str()).map(|p| RuntimeValue::String(p.to_string())).collect()
                };
                Ok(RuntimeValue::List(parts))
            }
            "join" => {
                let list = args.first().ok_or_else(|| RuntimeError::type_error("join() requires a list"))?;
                match list {
                    RuntimeValue::List(items) => {
                        let r = items.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(&s);
                        Ok(RuntimeValue::String(r))
                    }
                    _ => Err(RuntimeError::type_error("join() requires a list")),
                }
            }
            "replace" => {
                if args.len() < 2 { return Err(RuntimeError::type_error("replace() requires 2 arguments")); }
                let old = args[0].as_string().ok_or_else(|| RuntimeError::type_error("replace() args must be strings"))?;
                let new = args[1].as_string().ok_or_else(|| RuntimeError::type_error("replace() args must be strings"))?;
                Ok(RuntimeValue::String(s.replace(old, new)))
            }
            "contains" | "includes" => {
                let sub = args.first().and_then(|a| a.as_string())
                    .ok_or_else(|| RuntimeError::type_error("contains() requires a string"))?;
                Ok(RuntimeValue::Bool(s.contains(sub)))
            }
            "startswith" | "startsWith" => {
                let p = args.first().and_then(|a| a.as_string())
                    .ok_or_else(|| RuntimeError::type_error("startswith() requires a string"))?;
                Ok(RuntimeValue::Bool(s.starts_with(p)))
            }
            "endswith" | "endsWith" | "ends_with" => {
                let p = args.first().and_then(|a| a.as_string())
                    .ok_or_else(|| RuntimeError::type_error("endswith() requires a string"))?;
                Ok(RuntimeValue::Bool(s.ends_with(p)))
            }
            "find" | "indexOf" => {
                let sub = args.first().and_then(|a| a.as_string())
                    .ok_or_else(|| RuntimeError::type_error("find() requires a string"))?;
                Ok(RuntimeValue::Int(s.find(sub).map(|i| i as i64).unwrap_or(-1)))
            }
            "count" => {
                let sub = args.first().and_then(|a| a.as_string())
                    .ok_or_else(|| RuntimeError::type_error("count() requires a string"))?;
                Ok(RuntimeValue::Int(s.matches(sub).count() as i64))
            }
            "len" | "length" | "size" => Ok(RuntimeValue::Int(s.chars().count() as i64)),
            "isdigit" | "isDigit" => Ok(RuntimeValue::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))),
            "isalpha" | "isAlpha" => Ok(RuntimeValue::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphabetic()))),
            "isalnum" => Ok(RuntimeValue::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphanumeric()))),
            "isspace" => Ok(RuntimeValue::Bool(!s.is_empty() && s.chars().all(|c| c.is_whitespace()))),
            "isupper" => Ok(RuntimeValue::Bool(!s.is_empty() && s.chars().all(|c| c.is_uppercase()))),
            "islower" => Ok(RuntimeValue::Bool(!s.is_empty() && s.chars().all(|c| c.is_lowercase()))),
            "repeat" => {
                let n = args.first().and_then(|a| a.as_int())
                    .ok_or_else(|| RuntimeError::type_error("repeat() requires an integer"))?;
                Ok(RuntimeValue::String(s.repeat(n.max(0) as usize)))
            }
            "substring" | "slice" => {
                let start = args.first().and_then(|a| a.as_int()).unwrap_or(0) as usize;
                let end = args.get(1).and_then(|a| a.as_int()).unwrap_or(s.len() as i64) as usize;
                let chars: Vec<char> = s.chars().collect();
                let end = end.min(chars.len());
                let start = start.min(end);
                Ok(RuntimeValue::String(chars[start..end].iter().collect()))
            }
            "charAt" => {
                let idx = args.first().and_then(|a| a.as_int())
                    .ok_or_else(|| RuntimeError::type_error("charAt() requires an integer"))?;
                let ch = s.chars().nth(idx as usize)
                    .ok_or_else(|| RuntimeError::index_out_of_bounds(idx, s.len()))?;
                Ok(RuntimeValue::String(ch.to_string()))
            }
            _ => Err(RuntimeError::type_error(format!("'str' has no method '{}'", name))),
        }
    }

    // ── List method dispatch ──────────────────────────────────

    fn list_method(mut items: Vec<RuntimeValue>, name: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
        match name {
            "append" | "add" | "push" => {
                let item = args.first().cloned()
                    .ok_or_else(|| RuntimeError::type_error("append() requires 1 argument"))?;
                items.push(item);
                Ok(RuntimeValue::List(items))
            }
            "pop" => {
                if let Some(idx_val) = args.first() {
                    let i = idx_val.as_int().ok_or_else(|| RuntimeError::type_error("pop() index must be an integer"))?;
                    let real = if i < 0 { (items.len() as i64 + i) as usize } else { i as usize };
                    if real >= items.len() { return Err(RuntimeError::index_out_of_bounds(i, items.len())); }
                    Ok(items.remove(real))
                } else {
                    items.pop().ok_or_else(|| RuntimeError::type_error("pop() on empty list"))
                }
            }
            "insert" => {
                if args.len() < 2 { return Err(RuntimeError::type_error("insert(index, item) requires 2 arguments")); }
                let idx = args[0].as_int().ok_or_else(|| RuntimeError::type_error("insert() index must be integer"))? as usize;
                let item = args[1].clone();
                let idx = idx.min(items.len());
                items.insert(idx, item);
                Ok(RuntimeValue::List(items))
            }
            "remove" => {
                let item = args.first().ok_or_else(|| RuntimeError::type_error("remove() requires 1 argument"))?;
                if let Some(pos) = items.iter().position(|x| x == item) {
                    items.remove(pos);
                    Ok(RuntimeValue::List(items))
                } else {
                    Err(RuntimeError::type_error(format!("'{}' not in list", item)))
                }
            }
            "extend" => {
                match args.first() {
                    Some(RuntimeValue::List(other)) => { items.extend_from_slice(other); Ok(RuntimeValue::List(items)) }
                    _ => Err(RuntimeError::type_error("extend() requires a list")),
                }
            }
            "sort" => {
                items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Ok(RuntimeValue::List(items))
            }
            "reverse" => { items.reverse(); Ok(RuntimeValue::List(items)) }
            "clear" => Ok(RuntimeValue::List(Vec::new())),
            "copy" => Ok(RuntimeValue::List(items)),
            "index" | "indexOf" => {
                let item = args.first().ok_or_else(|| RuntimeError::type_error("index() requires 1 argument"))?;
                items.iter().position(|x| x == item)
                    .map(|i| RuntimeValue::Int(i as i64))
                    .ok_or_else(|| RuntimeError::type_error(format!("'{}' not in list", item)))
            }
            "count" => {
                let item = args.first().ok_or_else(|| RuntimeError::type_error("count() requires 1 argument"))?;
                Ok(RuntimeValue::Int(items.iter().filter(|x| *x == item).count() as i64))
            }
            "contains" | "includes" => {
                let item = args.first().ok_or_else(|| RuntimeError::type_error("contains() requires 1 argument"))?;
                Ok(RuntimeValue::Bool(items.contains(item)))
            }
            "len" | "length" | "size" => Ok(RuntimeValue::Int(items.len() as i64)),
            "first" | "head" => items.first().cloned().ok_or_else(|| RuntimeError::type_error("list is empty")),
            "last" => items.last().cloned().ok_or_else(|| RuntimeError::type_error("list is empty")),
            "isEmpty" | "is_empty" => Ok(RuntimeValue::Bool(items.is_empty())),
            "join" => {
                let sep = args.first().and_then(|a| a.as_string()).unwrap_or("").to_string();
                Ok(RuntimeValue::String(items.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(&sep)))
            }
            "get" => {
                let i = args.first().and_then(|a| a.as_int())
                    .ok_or_else(|| RuntimeError::type_error("get() requires an integer index"))?;
                let real = if i < 0 { (items.len() as i64 + i) as usize } else { i as usize };
                items.get(real).cloned().ok_or_else(|| RuntimeError::index_out_of_bounds(i, items.len()))
            }
            _ => Err(RuntimeError::type_error(format!("'list' has no method '{}'", name))),
        }
    }

    // ── Dict method dispatch ──────────────────────────────────

    fn dict_method(mut pairs: Vec<(RuntimeValue, RuntimeValue)>, name: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
        match name {
            "get" => {
                let key = args.first().ok_or_else(|| RuntimeError::type_error("get() requires a key"))?;
                let default = args.get(1).cloned().unwrap_or(RuntimeValue::Null);
                Ok(pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).unwrap_or(default))
            }
            "put" | "set" => {
                if args.len() < 2 { return Err(RuntimeError::type_error("put() requires key and value")); }
                let (key, value) = (args[0].clone(), args[1].clone());
                if let Some(p) = pairs.iter_mut().find(|(k, _)| k == &key) { p.1 = value; } else { pairs.push((key, value)); }
                Ok(RuntimeValue::Dict(pairs))
            }
            "remove" | "delete" => {
                let key = args.first().ok_or_else(|| RuntimeError::type_error("remove() requires a key"))?;
                pairs.retain(|(k, _)| k != key);
                Ok(RuntimeValue::Dict(pairs))
            }
            "keys" | "keySet" => Ok(RuntimeValue::List(pairs.into_iter().map(|(k, _)| k).collect())),
            "values" => Ok(RuntimeValue::List(pairs.into_iter().map(|(_, v)| v).collect())),
            "items" | "entries" | "entrySet" => Ok(RuntimeValue::List(
                pairs.into_iter().map(|(k, v)| RuntimeValue::List(vec![k, v])).collect(),
            )),
            "contains" | "containsKey" | "has" => {
                let key = args.first().ok_or_else(|| RuntimeError::type_error("containsKey() requires a key"))?;
                Ok(RuntimeValue::Bool(pairs.iter().any(|(k, _)| k == key)))
            }
            "len" | "length" | "size" => Ok(RuntimeValue::Int(pairs.len() as i64)),
            "isEmpty" | "is_empty" => Ok(RuntimeValue::Bool(pairs.is_empty())),
            "clear" => Ok(RuntimeValue::Dict(Vec::new())),
            _ => Err(RuntimeError::type_error(format!("'dict' has no method '{}'", name))),
        }
    }

    /// Call a registered builtin function by name.
    fn call_builtin(
        &mut self,
        name: &str,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue, RuntimeError> {
        // Special handling for print — capture output
        if name == "print" {
            let text = args
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(" ");
            println!("{}", text);
            if let Some(ref mut out) = self.output { out.push(text); }
            return Ok(RuntimeValue::Null);
        }

        // Special handling for serve — needs mutable VM access for callbacks
        if name == "serve" {
            if args.len() < 2 {
                return Err(RuntimeError::type_error("serve(port, handler) requires 2 arguments"));
            }
            let port = args[0]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("serve() port must be an integer"))? as u16;
            let func_idx = match &args[1] {
                RuntimeValue::Function(idx) => *idx,
                _ => return Err(RuntimeError::type_error("serve() handler must be a function")),
            };
            return self.run_http_server(port, func_idx);
        }

        if let Some(func) = self.builtins.get(name) {
            func(args)
        } else {
            Err(RuntimeError::type_error(format!(
                "unknown builtin function '{}'",
                name
            )))
        }
    }

    // ── HTTP server support ───────────────────────────────────

    /// Run an HTTP server on `port`, calling `func_idx` for every request.
    fn run_http_server(&mut self, port: u16, func_idx: usize) -> Result<RuntimeValue, RuntimeError> {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).map_err(|e| {
            RuntimeError::type_error(format!("serve(): cannot bind to {}: {}", addr, e))
        })?;

        println!("UniLang HTTP server listening on http://localhost:{}", port);
        println!("Press Ctrl+C to stop.");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = [0u8; 8192];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let raw = std::str::from_utf8(&buf[..n]).unwrap_or("").to_string();
                    let request = Self::parse_http_request(&raw);
                    let response = match self.call_unilang_function(func_idx, vec![request]) {
                        Ok(r) => r,
                        Err(e) => {
                            let body = format!("Internal Server Error: {}", e.message);
                            let _ = stream.write_all(
                                format!(
                                    "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    body.len(), body
                                )
                                .as_bytes(),
                            );
                            continue;
                        }
                    };
                    Self::write_http_response(&mut stream, response);
                }
                Err(_) => continue,
            }
        }

        Ok(RuntimeValue::Null)
    }

    /// Invoke a UniLang function by index with the given arguments.
    fn call_unilang_function(
        &mut self,
        func_idx: usize,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let func = self.functions[func_idx].clone();
        let n_args = args.len();
        let local_count = func.local_count.max(n_args);
        let mut locals = vec![RuntimeValue::Null; local_count];
        for (i, arg) in args.into_iter().enumerate() {
            if i < local_count {
                locals[i] = arg;
            }
        }

        let target_depth = self.frames.len();
        if self.frames.len() >= MAX_CALL_DEPTH {
            return Err(RuntimeError::new(
                ErrorKind::Exception,
                format!("maximum call depth exceeded ({})", MAX_CALL_DEPTH),
            ));
        }
        let frame = CallFrame {
            code: func.code.clone(),
            ip: 0,
            locals,
            stack_base: self.stack.len(),
        };
        self.frames.push(frame);

        loop {
            if self.frames.len() <= target_depth {
                break;
            }
            match self.step() {
                Ok(true) => continue,
                Ok(false) => break,
                Err(e) if e.kind == ErrorKind::Halt => break,
                Err(e) => {
                    while self.frames.len() > target_depth {
                        self.frames.pop();
                    }
                    return Err(e);
                }
            }
        }

        Ok(self.stack.pop().unwrap_or(RuntimeValue::Null))
    }

    /// Parse a raw HTTP request string into a Dict.
    fn parse_http_request(raw: &str) -> RuntimeValue {
        let mut lines = raw.lines();
        let (method, path) = if let Some(line) = lines.next() {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            (
                parts.first().copied().unwrap_or("GET").to_string(),
                parts.get(1).copied().unwrap_or("/").to_string(),
            )
        } else {
            ("GET".to_string(), "/".to_string())
        };

        // Strip query string from path for simplicity; expose full path.
        let clean_path = path.splitn(2, '?').next().unwrap_or(&path).to_string();
        let query = path.splitn(2, '?').nth(1).unwrap_or("").to_string();

        let mut header_pairs = Vec::new();
        let mut body = String::new();
        let mut in_body = false;
        for line in lines {
            if in_body {
                if !body.is_empty() { body.push('\n'); }
                body.push_str(line);
            } else if line.is_empty() {
                in_body = true;
            } else if let Some(colon) = line.find(':') {
                let key = line[..colon].trim().to_lowercase();
                let val = line[colon + 1..].trim().to_string();
                header_pairs.push((
                    RuntimeValue::String(key),
                    RuntimeValue::String(val),
                ));
            }
        }

        RuntimeValue::Dict(vec![
            (RuntimeValue::String("method".to_string()), RuntimeValue::String(method)),
            (RuntimeValue::String("path".to_string()), RuntimeValue::String(clean_path)),
            (RuntimeValue::String("query".to_string()), RuntimeValue::String(query)),
            (RuntimeValue::String("headers".to_string()), RuntimeValue::Dict(header_pairs)),
            (RuntimeValue::String("body".to_string()), RuntimeValue::String(body.trim().to_string())),
        ])
    }

    /// Write a RuntimeValue (String or Dict) as an HTTP/1.1 response.
    fn write_http_response<W: std::io::Write>(stream: &mut W, response: RuntimeValue) {
        let (status, body, content_type) = match response {
            RuntimeValue::Dict(ref pairs) => {
                let status = pairs
                    .iter()
                    .find(|(k, _)| k == &RuntimeValue::String("status".to_string()))
                    .map(|(_, v)| match v {
                        RuntimeValue::Int(n) => *n as u16,
                        _ => 200,
                    })
                    .unwrap_or(200);
                let body = pairs
                    .iter()
                    .find(|(k, _)| k == &RuntimeValue::String("body".to_string()))
                    .map(|(_, v)| format!("{}", v))
                    .unwrap_or_default();
                let ct = pairs
                    .iter()
                    .find(|(k, _)| k == &RuntimeValue::String("content_type".to_string()))
                    .map(|(_, v)| format!("{}", v))
                    .unwrap_or_else(|| "application/json".to_string());
                (status, body, ct)
            }
            RuntimeValue::String(s) => (200, s, "text/plain".to_string()),
            other => (200, format!("{}", other), "text/plain".to_string()),
        };

        let status_text = match status {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        };

        let response_str = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
            status, status_text, content_type, body.len(), body
        );
        let _ = stream.write_all(response_str.as_bytes());
    }

    /// Helper for numeric binary operations.
    fn numeric_op(
        &self,
        a: &RuntimeValue,
        b: &RuntimeValue,
        int_op: impl Fn(i64, i64) -> i64,
        float_op: impl Fn(f64, f64) -> f64,
        op_name: &str,
    ) -> Result<RuntimeValue, RuntimeError> {
        match (a, b) {
            (RuntimeValue::Int(x), RuntimeValue::Int(y)) => Ok(RuntimeValue::Int(int_op(*x, *y))),
            (RuntimeValue::Float(x), RuntimeValue::Float(y)) => {
                Ok(RuntimeValue::Float(float_op(*x, *y)))
            }
            (RuntimeValue::Int(x), RuntimeValue::Float(y)) => {
                Ok(RuntimeValue::Float(float_op(*x as f64, *y)))
            }
            (RuntimeValue::Float(x), RuntimeValue::Int(y)) => {
                Ok(RuntimeValue::Float(float_op(*x, *y as f64)))
            }
            _ => Err(RuntimeError::type_error(format!(
                "cannot {} {} and {}",
                op_name, a, b
            ))),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
