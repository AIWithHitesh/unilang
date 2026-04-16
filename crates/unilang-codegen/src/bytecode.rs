// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Stack-based bytecode instruction set and container types.
//!
//! Defines the IR that the UniLang compiler emits and that a
//! simple stack-based interpreter can execute.

/// A runtime value that can appear as an instruction operand.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

/// Stack-based bytecode instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    // ── Stack manipulation ───────────────────────────────
    /// Push a constant value onto the stack.
    LoadConst(Value),
    /// Push local variable by slot index.
    LoadLocal(usize),
    /// Pop stack top and store into local slot.
    StoreLocal(usize),
    /// Push global variable by name.
    LoadGlobal(String),
    /// Pop stack top and store into global.
    StoreGlobal(String),
    /// Discard the top of the stack.
    Pop,
    /// Duplicate the top of the stack.
    Dup,

    // ── Arithmetic ───────────────────────────────────────
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Neg,

    // ── Comparison ───────────────────────────────────────
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // ── Logical ──────────────────────────────────────────
    And,
    Or,
    Not,

    // ── Bitwise ──────────────────────────────────────────
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    LShift,
    RShift,

    // ── String ───────────────────────────────────────────
    /// Concatenate two strings from the stack.
    Concat,

    // ── Control flow ─────────────────────────────────────
    /// Unconditional jump to instruction index.
    Jump(usize),
    /// Pop top; jump if falsy.
    JumpIfFalse(usize),
    /// Pop top; jump if truthy.
    JumpIfTrue(usize),

    // ── Functions ────────────────────────────────────────
    /// Call function with N arguments on the stack.
    Call(usize),
    /// Return from function (top of stack is return value).
    Return,
    /// Create a function object (index into function table).
    MakeFunction(usize),

    // ── Objects / classes ────────────────────────────────
    /// Get attribute from object on stack top.
    GetAttr(String),
    /// Set attribute: stack = [obj, value].
    SetAttr(String),
    /// Create a class with N methods/fields.
    MakeClass(String, usize),
    /// Instantiate a class by name.
    NewInstance(String),

    // ── Collections ──────────────────────────────────────
    /// Create a list from N stack items.
    MakeList(usize),
    /// Create a dict from N key-value pairs (2N items on stack).
    MakeDict(usize),
    /// Index access: `stack[index]`.
    GetIndex,
    /// Index assignment: `stack[index] = value`.
    SetIndex,

    // ── Built-ins ────────────────────────────────────────
    /// Pop and print the top of the stack.
    Print,

    // ── Method calls ─────────────────────────────────────
    /// Call method `name` on receiver with N args.
    /// Stack: [receiver, arg0, ..., argN-1] → [return_value]
    CallMethod(String, usize),

    // ── Membership test ──────────────────────────────────
    /// `x in container`: stack [item, container] → Bool
    Contains,

    // ── Assertions & exceptions ──────────────────────────
    /// `assert expr, msg`: stack [condition, msg] → () or RuntimeError
    Assert,

    /// `raise/throw expr`: stack [exception_value] → RuntimeError
    Raise,

    /// Register an exception handler: if any error occurs before `PopExceptHandler`,
    /// restore the stack to current depth, push the error message, and jump to `catch_ip`.
    PushExceptHandler(usize),

    /// Remove the most-recently-registered exception handler (no error occurred).
    PopExceptHandler,

    /// Pop the exception value from the stack and store it in a named global.
    /// Emitted as the first instruction of every catch block.
    StoreExceptVar(String),

    // ── Halt ─────────────────────────────────────────────
    /// Stop execution.
    Halt,
}

/// A compiled function.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub code: Vec<Opcode>,
    pub local_count: usize,
}

/// A compiled class definition.
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    /// Indices into the `Bytecode::functions` table.
    pub methods: Vec<usize>,
    pub fields: Vec<String>,
}

/// Top-level bytecode container produced by the compiler.
#[derive(Debug, Clone)]
pub struct Bytecode {
    /// Module-level (top-level) instructions.
    pub instructions: Vec<Opcode>,
    /// Function table.
    pub functions: Vec<Function>,
    /// Class definitions.
    pub classes: Vec<ClassDef>,
}

impl Bytecode {
    /// Create an empty bytecode container.
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}
