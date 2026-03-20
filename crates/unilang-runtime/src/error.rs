// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Runtime error types for the UniLang virtual machine.

use std::fmt;

/// The kind of runtime error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    TypeError,
    DivisionByZero,
    UndefinedVariable,
    StackUnderflow,
    IndexOutOfBounds,
    AttributeError,
    /// Not a real error -- signals normal program termination via `Halt`.
    Halt,
}

/// A runtime error produced by the VM.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub kind: ErrorKind,
}

impl RuntimeError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }

    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::TypeError, msg)
    }

    pub fn division_by_zero() -> Self {
        Self::new(ErrorKind::DivisionByZero, "division by zero")
    }

    pub fn undefined_variable(name: &str) -> Self {
        Self::new(
            ErrorKind::UndefinedVariable,
            format!("undefined variable '{}'", name),
        )
    }

    pub fn stack_underflow() -> Self {
        Self::new(ErrorKind::StackUnderflow, "stack underflow")
    }

    pub fn index_out_of_bounds(index: i64, len: usize) -> Self {
        Self::new(
            ErrorKind::IndexOutOfBounds,
            format!("index {} out of bounds for length {}", index, len),
        )
    }

    pub fn attribute_error(name: &str) -> Self {
        Self::new(
            ErrorKind::AttributeError,
            format!("attribute '{}' not found", name),
        )
    }

    pub fn halt() -> Self {
        Self::new(ErrorKind::Halt, "program halted")
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
