// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang bytecode code generator.
//!
//! Transforms the unified AST into a stack-based bytecode IR
//! that can be executed by a simple interpreter.

pub mod bytecode;
pub mod compiler;

use unilang_common::error::Diagnostic;
use unilang_parser::ast::Module;

use crate::bytecode::Bytecode;
use crate::compiler::Compiler;

/// Compile a parsed [`Module`] into [`Bytecode`].
///
/// Returns `Ok(Bytecode)` on success, or `Err(diagnostics)` if
/// code generation encountered fatal errors.
pub fn compile(module: &Module) -> Result<Bytecode, Vec<Diagnostic>> {
    let compiler = Compiler::new();
    compiler.compile_module(module)
}

#[cfg(test)]
mod tests;
