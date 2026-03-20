// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # UniLang Parser
//!
//! Parses a token stream into a unified AST that represents both Python
//! and Java syntax. Uses a context-stack approach for disambiguation
//! and a Pratt parser for expressions.

pub mod ast;
pub mod expr;
pub mod parser;
pub mod stmt;
pub mod types;

pub use ast::Module;
pub use parser::Parser;

use unilang_common::error::DiagnosticBag;
use unilang_common::span::SourceId;
use unilang_lexer::Lexer;

/// Convenience function: lex and parse a source string.
pub fn parse(source_id: SourceId, source: &str) -> (Module, DiagnosticBag) {
    let lexer = Lexer::new(source_id, source);
    let (tokens, _lexer_diag) = lexer.tokenize();
    let mut parser = Parser::new(tokens, source, source_id);
    let module = parser.parse();
    (module, parser.diagnostics())
}
