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
    /// Captured print output (for testing).
    output: Vec<String>,
    /// Registered builtin functions.
    builtins: HashMap<String, BuiltinFn>,
}

impl VM {
    /// Create a new VM.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            output: Vec::new(),
            builtins: HashMap::new(),
        }
    }

    /// Return captured print output.
    pub fn output(&self) -> &[String] {
        &self.output
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

        self.execute(instr)
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
                        self.output.push(text);
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

            Opcode::MakeClass(_name, _member_count) => {
                // Class definitions are registered at bytecode load time.
                // This opcode is a no-op at runtime for now.
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
                self.output.push(text);
            }

            // ── Halt ─────────────────────────────────────────
            Opcode::Halt => {
                return Err(RuntimeError::halt());
            }
        }

        Ok(true)
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
            self.output.push(text);
            return Ok(RuntimeValue::Null);
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
