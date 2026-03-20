// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Diagnostic generation for UniLang source files.
//!
//! Runs the lexer on the source text and converts any errors into
//! LSP-compatible diagnostics. Also performs basic structural checks
//! such as unmatched brackets, braces, and parentheses.

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range,
};
use unilang_common::error::Severity;
use unilang_common::span::SourceId;
use unilang_lexer::token::TokenKind;
use unilang_lexer::Lexer;

/// Generate LSP diagnostics for the given source text.
pub fn generate_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Run the lexer and collect its diagnostics.
    let lexer = Lexer::new(SourceId(0), source);
    let (tokens, diag_bag) = lexer.tokenize();

    // Convert lexer diagnostics to LSP diagnostics.
    for diag in diag_bag.diagnostics() {
        let severity = match diag.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Hint => DiagnosticSeverity::HINT,
        };

        // Use the first label's span for position, or fall back to (0,0).
        let range = if let Some(label) = diag.labels.first() {
            byte_span_to_range(source, label.span.start, label.span.end)
        } else {
            Range::new(Position::new(0, 0), Position::new(0, 0))
        };

        diagnostics.push(Diagnostic {
            range,
            severity: Some(severity),
            code: diag.code.as_ref().map(|c| NumberOrString::String(c.clone())),
            code_description: None,
            source: Some("unilang".to_string()),
            message: diag.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        });
    }

    // Check for error tokens in the token stream.
    for token in &tokens {
        if token.kind == TokenKind::Error {
            let range = byte_span_to_range(source, token.span.start, token.span.end);
            // Avoid duplicating diagnostics already reported by the lexer.
            let already_reported = diagnostics.iter().any(|d| d.range == range);
            if !already_reported {
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("E0001".to_string())),
                    code_description: None,
                    source: Some("unilang".to_string()),
                    message: "unexpected token".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }
    }

    // Structural checks: unmatched brackets, braces, parentheses.
    check_balanced_delimiters(source, &tokens, &mut diagnostics);

    diagnostics
}

/// Check for unmatched delimiters and report diagnostics.
fn check_balanced_delimiters(
    source: &str,
    tokens: &[unilang_lexer::token::Token],
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Stack of (delimiter kind, byte position).
    let mut stack: Vec<(TokenKind, u32)> = Vec::new();

    for token in tokens {
        match token.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                stack.push((token.kind, token.span.start));
            }
            TokenKind::RParen => {
                if let Some((open, _)) = stack.last() {
                    if *open == TokenKind::LParen {
                        stack.pop();
                    } else {
                        let range =
                            byte_span_to_range(source, token.span.start, token.span.end);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("E0104".to_string())),
                            code_description: None,
                            source: Some("unilang".to_string()),
                            message: "mismatched closing ')'".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    }
                } else {
                    let range =
                        byte_span_to_range(source, token.span.start, token.span.end);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("E0104".to_string())),
                        code_description: None,
                        source: Some("unilang".to_string()),
                        message: "unmatched closing ')'".to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            TokenKind::RBracket => {
                if let Some((open, _)) = stack.last() {
                    if *open == TokenKind::LBracket {
                        stack.pop();
                    } else {
                        let range =
                            byte_span_to_range(source, token.span.start, token.span.end);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("E0105".to_string())),
                            code_description: None,
                            source: Some("unilang".to_string()),
                            message: "mismatched closing ']'".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    }
                } else {
                    let range =
                        byte_span_to_range(source, token.span.start, token.span.end);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("E0105".to_string())),
                        code_description: None,
                        source: Some("unilang".to_string()),
                        message: "unmatched closing ']'".to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            TokenKind::RBrace => {
                if let Some((open, _)) = stack.last() {
                    if *open == TokenKind::LBrace {
                        stack.pop();
                    } else {
                        let range =
                            byte_span_to_range(source, token.span.start, token.span.end);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("E0103".to_string())),
                            code_description: None,
                            source: Some("unilang".to_string()),
                            message: "mismatched closing '}'".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    }
                } else {
                    let range =
                        byte_span_to_range(source, token.span.start, token.span.end);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("E0103".to_string())),
                        code_description: None,
                        source: Some("unilang".to_string()),
                        message: "unmatched closing '}'".to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            _ => {}
        }
    }

    // Report any unclosed openers remaining on the stack.
    for (kind, start_pos) in stack {
        let (code, name) = match kind {
            TokenKind::LParen => ("E0104", "("),
            TokenKind::LBracket => ("E0105", "["),
            TokenKind::LBrace => ("E0103", "{"),
            _ => continue,
        };
        let range = byte_span_to_range(source, start_pos, start_pos + 1);
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.to_string())),
            code_description: None,
            source: Some("unilang".to_string()),
            message: format!("unclosed '{}'", name),
            related_information: None,
            tags: None,
            data: None,
        });
    }
}

/// Convert a byte offset span to an LSP `Range` (line/character positions).
fn byte_span_to_range(source: &str, start: u32, end: u32) -> Range {
    let start_pos = byte_offset_to_position(source, start as usize);
    let end_pos = byte_offset_to_position(source, end as usize);
    Range::new(start_pos, end_pos)
}

/// Convert a byte offset into an LSP `Position` (0-indexed line and character).
fn byte_offset_to_position(source: &str, offset: usize) -> Position {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.chars().filter(|&c| c == '\n').count() as u32;
    let last_newline = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col = source[last_newline..offset].chars().count() as u32;
    Position::new(line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_diagnostics_on_valid_code() {
        let diags = generate_diagnostics("x = 42");
        assert!(diags.is_empty(), "Expected no diagnostics, got: {:?}", diags);
    }

    #[test]
    fn test_unclosed_paren() {
        let diags = generate_diagnostics("foo(x, y");
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("unclosed")));
    }

    #[test]
    fn test_byte_offset_to_position_first_line() {
        let pos = byte_offset_to_position("hello world", 6);
        assert_eq!(pos, Position::new(0, 6));
    }

    #[test]
    fn test_byte_offset_to_position_second_line() {
        let pos = byte_offset_to_position("hello\nworld", 8);
        assert_eq!(pos, Position::new(1, 2));
    }
}
