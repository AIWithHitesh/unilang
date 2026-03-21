// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Runtime value representation for the UniLang virtual machine.

use std::collections::HashMap;
use std::fmt;

use unilang_codegen::bytecode::Value;

/// A runtime value on the VM stack.
#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<RuntimeValue>),
    Dict(Vec<(RuntimeValue, RuntimeValue)>),
    /// Index into the function table.
    Function(usize),
    /// An object instance.
    Instance(InstanceData),
    /// A native (built-in) function, identified by name.
    NativeFunction(std::string::String),
}

/// Data for an instantiated class.
#[derive(Debug, Clone)]
pub struct InstanceData {
    pub class_name: String,
    pub fields: HashMap<String, RuntimeValue>,
}

impl RuntimeValue {
    /// Whether this value is considered truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Int(n) => *n != 0,
            RuntimeValue::Float(f) => *f != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Null => false,
            RuntimeValue::List(items) => !items.is_empty(),
            RuntimeValue::Dict(pairs) => !pairs.is_empty(),
            RuntimeValue::Function(_) => true,
            RuntimeValue::Instance(_) => true,
            RuntimeValue::NativeFunction(_) => true,
        }
    }

    /// Try to extract an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            RuntimeValue::Int(n) => Some(*n),
            RuntimeValue::Float(f) => Some(*f as i64),
            RuntimeValue::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Try to extract a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            RuntimeValue::Float(f) => Some(*f),
            RuntimeValue::Int(n) => Some(*n as f64),
            RuntimeValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Try to extract a string reference.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            RuntimeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Coerce this value to an integer, returning a RuntimeError on failure.
    pub fn coerce_to_int(&self) -> Result<i64, crate::error::RuntimeError> {
        match self {
            RuntimeValue::Int(n) => Ok(*n),
            RuntimeValue::Float(f) => Ok(*f as i64),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::String(s) => s.parse::<i64>().map_err(|_| {
                crate::error::RuntimeError::type_error(format!("cannot convert '{}' to int", s))
            }),
            RuntimeValue::Null => Ok(0),
            other => Err(crate::error::RuntimeError::type_error(format!(
                "cannot convert {} to int",
                other
            ))),
        }
    }

    /// Coerce this value to a float, returning a RuntimeError on failure.
    pub fn coerce_to_float(&self) -> Result<f64, crate::error::RuntimeError> {
        match self {
            RuntimeValue::Float(f) => Ok(*f),
            RuntimeValue::Int(n) => Ok(*n as f64),
            RuntimeValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            RuntimeValue::String(s) => s.parse::<f64>().map_err(|_| {
                crate::error::RuntimeError::type_error(format!("cannot convert '{}' to float", s))
            }),
            RuntimeValue::Null => Ok(0.0),
            other => Err(crate::error::RuntimeError::type_error(format!(
                "cannot convert {} to float",
                other
            ))),
        }
    }

    /// Coerce this value to a string. Always succeeds.
    pub fn coerce_to_string(&self) -> String {
        format!("{}", self)
    }

    /// Coerce this value to a boolean (truthiness). Always succeeds.
    pub fn coerce_to_bool(&self) -> bool {
        self.is_truthy()
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Int(n) => write!(f, "{}", n),
            RuntimeValue::Float(v) => {
                if v.fract() == 0.0 {
                    write!(f, "{:.1}", v)
                } else {
                    write!(f, "{}", v)
                }
            }
            RuntimeValue::String(s) => write!(f, "{}", s),
            RuntimeValue::Bool(b) => {
                if *b {
                    write!(f, "True")
                } else {
                    write!(f, "False")
                }
            }
            RuntimeValue::Null => write!(f, "None"),
            RuntimeValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            RuntimeValue::Dict(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            RuntimeValue::Function(idx) => write!(f, "<function {}>", idx),
            RuntimeValue::Instance(data) => write!(f, "<{} instance>", data.class_name),
            RuntimeValue::NativeFunction(name) => write!(f, "<builtin {}>", name),
        }
    }
}

impl From<Value> for RuntimeValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(n) => RuntimeValue::Int(n),
            Value::Float(f) => RuntimeValue::Float(f),
            Value::String(s) => RuntimeValue::String(s),
            Value::Bool(b) => RuntimeValue::Bool(b),
            Value::Null => RuntimeValue::Null,
        }
    }
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a == b,
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a == b,
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => (*a as f64) == *b,
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => *a == (*b as f64),
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            (RuntimeValue::Null, RuntimeValue::Null) => true,
            (RuntimeValue::List(a), RuntimeValue::List(b)) => a == b,
            (RuntimeValue::Dict(a), RuntimeValue::Dict(b)) => a == b,
            (RuntimeValue::Function(a), RuntimeValue::Function(b)) => a == b,
            (RuntimeValue::NativeFunction(a), RuntimeValue::NativeFunction(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for RuntimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a.partial_cmp(b),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a.partial_cmp(b),
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => (*a as f64).partial_cmp(b),
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => a.partial_cmp(&(*b as f64)),
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a.partial_cmp(b),
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
