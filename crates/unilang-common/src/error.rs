// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

use crate::span::{SourceId, Span};

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Hint,
}

/// A label pointing to a span of source code within a diagnostic.
#[derive(Debug, Clone)]
pub struct DiagnosticLabel {
    pub span: Span,
    pub source_id: SourceId,
    pub message: String,
}

/// A compiler diagnostic (error, warning, or hint).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: Option<String>,
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<DiagnosticLabel>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            code: None,
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            code: None,
            severity: Severity::Warning,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_label(
        mut self,
        span: Span,
        source_id: SourceId,
        message: impl Into<String>,
    ) -> Self {
        self.labels.push(DiagnosticLabel {
            span,
            source_id,
            message: message.into(),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

/// Accumulates diagnostics during compilation.
#[derive(Debug, Default)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

// Error code ranges:
// E0001–E0099: Lexer errors
// E0100–E0199: Parser errors
// E0200–E0299: Semantic errors
// E0300–E0399: Type errors
// E0400–E0499: Import resolution errors

pub mod codes {
    // Lexer
    pub const UNEXPECTED_CHAR: &str = "E0001";
    pub const UNTERMINATED_STRING: &str = "E0002";
    pub const UNTERMINATED_COMMENT: &str = "E0003";
    pub const INVALID_NUMBER: &str = "E0004";
    pub const MIXED_INDENTATION: &str = "E0005";
    pub const INCONSISTENT_INDENT: &str = "E0006";
    pub const UNTERMINATED_FSTRING: &str = "E0007";

    // Parser
    pub const UNEXPECTED_TOKEN: &str = "E0100";
    pub const EXPECTED_EXPRESSION: &str = "E0101";
    pub const EXPECTED_BLOCK: &str = "E0102";
    pub const UNCLOSED_BRACE: &str = "E0103";
    pub const UNCLOSED_PAREN: &str = "E0104";
    pub const UNCLOSED_BRACKET: &str = "E0105";
    pub const EXPECTED_IDENTIFIER: &str = "E0106";
    pub const EXPECTED_TYPE: &str = "E0107";
    pub const AMBIGUOUS_SYNTAX: &str = "E0108";
}
