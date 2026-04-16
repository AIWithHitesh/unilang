// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Type checking utilities for the semantic analyzer.
//!
//! Checks binary operators, function call arity, and assignment compatibility.
//! Supports gradual typing: if either side is `Dynamic`, the check passes.

use unilang_common::error::{Diagnostic, DiagnosticBag};
use unilang_common::span::{SourceId, Span};
use unilang_parser::ast::BinOp;

use crate::types::Type;

/// Check that a binary operator is valid for the given operand types.
/// Returns the result type, or `Type::Error` if invalid (with a diagnostic emitted).
pub fn check_binary_op(
    op: BinOp,
    left: &Type,
    right: &Type,
    span: Span,
    source_id: SourceId,
    diagnostics: &mut DiagnosticBag,
) -> Type {
    // Dynamic/Error/Unknown always pass
    if matches!(left, Type::Dynamic | Type::Error | Type::Unknown)
        || matches!(right, Type::Dynamic | Type::Error | Type::Unknown)
    {
        return Type::Dynamic;
    }

    match op {
        // Arithmetic operators
        BinOp::Add => {
            // String concatenation: String + anything → String (auto-convert to string)
            if matches!(left, Type::String) || matches!(right, Type::String) {
                Type::String
            } else if left.is_numeric() && right.is_numeric() {
                Type::coercion_result(left, right)
            } else {
                diagnostics.report(
                    Diagnostic::error(format!(
                        "operator '+' cannot be applied to '{}' and '{}'",
                        left.display_name(),
                        right.display_name()
                    ))
                    .with_code("E0300")
                    .with_label(span, source_id, "incompatible types for '+'"),
                );
                Type::Error
            }
        }
        BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::FloorDiv | BinOp::Mod | BinOp::Pow => {
            let left_ok  = left.is_numeric()  || matches!(left,  Type::Dynamic | Type::Unknown);
            let right_ok = right.is_numeric() || matches!(right, Type::Dynamic | Type::Unknown);
            if left_ok && right_ok {
                Type::coercion_result(left, right)
            } else {
                diagnostics.report(
                    Diagnostic::error(format!(
                        "arithmetic operator requires numeric types, got '{}' and '{}'",
                        left.display_name(),
                        right.display_name()
                    ))
                    .with_code("E0300")
                    .with_label(span, source_id, "non-numeric operands"),
                );
                Type::Error
            }
        }

        // Comparison operators — allow cross-type numeric comparison
        BinOp::Eq | BinOp::NotEq => Type::Bool,
        BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
            if (left.is_numeric() && right.is_numeric())
                || (matches!(left, Type::String) && matches!(right, Type::String))
            {
                Type::Bool
            } else {
                diagnostics.report(
                    Diagnostic::error(format!(
                        "comparison requires compatible types, got '{}' and '{}'",
                        left.display_name(),
                        right.display_name()
                    ))
                    .with_code("E0300")
                    .with_label(
                        span,
                        source_id,
                        "incompatible types for comparison",
                    ),
                );
                Type::Error
            }
        }

        // Logical operators
        BinOp::And | BinOp::Or => Type::Bool,

        // Bitwise operators
        BinOp::BitAnd
        | BinOp::BitOr
        | BinOp::BitXor
        | BinOp::LShift
        | BinOp::RShift
        | BinOp::UnsignedRShift => {
            if matches!(left, Type::Int) && matches!(right, Type::Int) {
                Type::Int
            } else {
                diagnostics.report(
                    Diagnostic::error(format!(
                        "bitwise operator requires integer types, got '{}' and '{}'",
                        left.display_name(),
                        right.display_name()
                    ))
                    .with_code("E0300")
                    .with_label(
                        span,
                        source_id,
                        "non-integer operands for bitwise op",
                    ),
                );
                Type::Error
            }
        }

        // Membership / identity
        BinOp::In | BinOp::NotIn | BinOp::Is | BinOp::IsNot | BinOp::Instanceof => Type::Bool,

        // Null coalesce
        BinOp::NullCoalesce => left.clone(),
    }
}

/// Check that a function call has the right number of arguments.
pub fn check_call_arity(
    param_count: usize,
    arg_count: usize,
    span: Span,
    source_id: SourceId,
    diagnostics: &mut DiagnosticBag,
) {
    if arg_count != param_count {
        diagnostics.report(
            Diagnostic::error(format!(
                "expected {} argument(s) but got {}",
                param_count, arg_count
            ))
            .with_code("E0301")
            .with_label(span, source_id, "wrong number of arguments"),
        );
    }
}

/// Check that a value type is assignable to a target type.
/// Returns `true` if the assignment is valid.
pub fn check_assignment_type(
    target: &Type,
    value: &Type,
    span: Span,
    source_id: SourceId,
    diagnostics: &mut DiagnosticBag,
) -> bool {
    if target.is_assignable_from(value) {
        return true;
    }
    diagnostics.report(
        Diagnostic::error(format!(
            "cannot assign '{}' to '{}'",
            value.display_name(),
            target.display_name()
        ))
        .with_code("E0302")
        .with_label(span, source_id, "type mismatch"),
    );
    false
}
