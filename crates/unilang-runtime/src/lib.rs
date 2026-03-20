// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang runtime: a stack-based virtual machine that interprets
//! the bytecodes produced by `unilang-codegen`.

pub mod error;
pub mod value;
pub mod vm;

use unilang_codegen::bytecode::Bytecode;

use crate::error::RuntimeError;
use crate::value::RuntimeValue;
use crate::vm::VM;

/// Execute bytecode and return the result.
///
/// This is the main entry point for running compiled UniLang programs.
pub fn execute(bytecode: &Bytecode) -> Result<RuntimeValue, RuntimeError> {
    let mut vm = VM::new();
    vm.run(bytecode)
}

/// Execute bytecode and also return captured print output.
pub fn execute_with_output(
    bytecode: &Bytecode,
) -> Result<(RuntimeValue, Vec<String>), RuntimeError> {
    let mut vm = VM::new();
    let result = vm.run(bytecode)?;
    let output = vm.output().to_vec();
    Ok((result, output))
}

#[cfg(test)]
mod tests;
