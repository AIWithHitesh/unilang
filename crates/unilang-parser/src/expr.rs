// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Pratt expression parser for UniLang.

use unilang_common::span::{Span, Spanned};
use unilang_lexer::token::TokenKind;

use crate::ast::*;
use crate::parser::Parser;

/// Precedence levels (lowest to highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum Prec {
    None = 0,
    Assignment = 1,
    Ternary = 2,
    Or = 3,
    And = 4,
    Not = 5,
    Comparison = 6,
    BitOr = 7,
    BitXor = 8,
    BitAnd = 9,
    Shift = 10,
    AddSub = 11,
    MulDiv = 12,
    Unary = 13,
    Postfix = 14,
    #[allow(dead_code)]
    Primary = 15,
}

/// Get the infix precedence of a token.
fn infix_prec(kind: TokenKind) -> Option<(Prec, BinOp)> {
    let pair = match kind {
        // Assignment
        TokenKind::Eq => return None, // handled specially
        TokenKind::PlusEq
        | TokenKind::MinusEq
        | TokenKind::StarEq
        | TokenKind::SlashEq
        | TokenKind::DoubleSlashEq
        | TokenKind::PercentEq
        | TokenKind::DoubleStarEq
        | TokenKind::AmpEq
        | TokenKind::PipeEq
        | TokenKind::CaretEq
        | TokenKind::LShiftEq
        | TokenKind::RShiftEq => return None, // handled as assignment

        // Or
        TokenKind::KwOr | TokenKind::PipePipe => (Prec::Or, BinOp::Or),
        // And
        TokenKind::KwAnd | TokenKind::AmpAmp => (Prec::And, BinOp::And),
        // Comparison
        TokenKind::EqEq => (Prec::Comparison, BinOp::Eq),
        TokenKind::NotEq => (Prec::Comparison, BinOp::NotEq),
        TokenKind::Lt => (Prec::Comparison, BinOp::Lt),
        TokenKind::Gt => (Prec::Comparison, BinOp::Gt),
        TokenKind::LtEq => (Prec::Comparison, BinOp::LtEq),
        TokenKind::GtEq => (Prec::Comparison, BinOp::GtEq),
        TokenKind::KwIn => (Prec::Comparison, BinOp::In),
        TokenKind::KwIs => (Prec::Comparison, BinOp::Is),
        TokenKind::KwInstanceof => (Prec::Comparison, BinOp::Instanceof),
        // BitOr
        TokenKind::Pipe => (Prec::BitOr, BinOp::BitOr),
        // BitXor
        TokenKind::Caret => (Prec::BitXor, BinOp::BitXor),
        // BitAnd
        TokenKind::Amp => (Prec::BitAnd, BinOp::BitAnd),
        // Shift
        TokenKind::LShift => (Prec::Shift, BinOp::LShift),
        TokenKind::RShift => (Prec::Shift, BinOp::RShift),
        TokenKind::UnsignedRShift => (Prec::Shift, BinOp::UnsignedRShift),
        // Add/Sub
        TokenKind::Plus => (Prec::AddSub, BinOp::Add),
        TokenKind::Minus => (Prec::AddSub, BinOp::Sub),
        // Mul/Div/Mod
        TokenKind::Star => (Prec::MulDiv, BinOp::Mul),
        TokenKind::Slash => (Prec::MulDiv, BinOp::Div),
        TokenKind::DoubleSlash => (Prec::MulDiv, BinOp::FloorDiv),
        TokenKind::Percent => (Prec::MulDiv, BinOp::Mod),
        TokenKind::DoubleStar => (Prec::MulDiv, BinOp::Pow),
        // NullCoalesce
        TokenKind::DoubleQuestion => (Prec::Or, BinOp::NullCoalesce),
        _ => return None,
    };
    Some(pair)
}

fn is_assignment_op(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Eq
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::DoubleSlashEq
            | TokenKind::PercentEq
            | TokenKind::DoubleStarEq
            | TokenKind::AmpEq
            | TokenKind::PipeEq
            | TokenKind::CaretEq
            | TokenKind::LShiftEq
            | TokenKind::RShiftEq
    )
}

fn assignment_to_binop(kind: TokenKind) -> Option<BinOp> {
    match kind {
        TokenKind::PlusEq => Some(BinOp::Add),
        TokenKind::MinusEq => Some(BinOp::Sub),
        TokenKind::StarEq => Some(BinOp::Mul),
        TokenKind::SlashEq => Some(BinOp::Div),
        TokenKind::DoubleSlashEq => Some(BinOp::FloorDiv),
        TokenKind::PercentEq => Some(BinOp::Mod),
        TokenKind::DoubleStarEq => Some(BinOp::Pow),
        TokenKind::AmpEq => Some(BinOp::BitAnd),
        TokenKind::PipeEq => Some(BinOp::BitOr),
        TokenKind::CaretEq => Some(BinOp::BitXor),
        TokenKind::LShiftEq => Some(BinOp::LShift),
        TokenKind::RShiftEq => Some(BinOp::RShift),
        _ => None,
    }
}

/// Parse an expression at the given minimum precedence.
pub(crate) fn parse_expr(p: &mut Parser<'_>, min_prec: Prec) -> Spanned<Expr> {
    let mut left = parse_prefix(p);

    loop {
        // Skip newlines inside expressions only if they are clearly continuations
        // (we don't skip newlines in general — the lexer handles ASI)

        let kind = p.peek_kind();

        // Handle assignment (right-associative, lowest precedence)
        if min_prec <= Prec::Assignment && is_assignment_op(kind) {
            let op_kind = kind;
            p.advance();
            let right = parse_expr(p, Prec::Assignment);
            let span = left.span.merge(right.span);

            if op_kind == TokenKind::Eq {
                left = Spanned::new(Expr::Assign(Box::new(left), Box::new(right)), span);
            } else if let Some(bin_op) = assignment_to_binop(op_kind) {
                // Desugar `a += b` to `a = a + b` (conceptually, but we keep it as BinaryOp + Assign)
                let bin_expr = Spanned::new(
                    Expr::BinaryOp(Box::new(left.clone()), bin_op, Box::new(right)),
                    span,
                );
                left = Spanned::new(Expr::Assign(Box::new(left), Box::new(bin_expr)), span);
            }
            continue;
        }

        // Handle `not in` and `is not`
        if kind == TokenKind::KwNot
            && p.peek_nth(1) == TokenKind::KwIn
            && min_prec <= Prec::Comparison
        {
            p.advance(); // not
            p.advance(); // in
            let right = parse_expr(p, Prec::Shift); // next higher prec
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::BinaryOp(Box::new(left), BinOp::NotIn, Box::new(right)),
                span,
            );
            continue;
        }

        if kind == TokenKind::KwIs
            && p.peek_nth(1) == TokenKind::KwNot
            && min_prec <= Prec::Comparison
        {
            p.advance(); // is
            p.advance(); // not
            let right = parse_expr(p, Prec::Shift);
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::BinaryOp(Box::new(left), BinOp::IsNot, Box::new(right)),
                span,
            );
            continue;
        }

        // Python ternary: `x if cond else y`
        if kind == TokenKind::KwIf && min_prec <= Prec::Ternary {
            p.advance(); // if
            let condition = parse_expr(p, Prec::Or);
            p.expect(TokenKind::KwElse);
            let else_expr = parse_expr(p, Prec::Ternary);
            let span = left.span.merge(else_expr.span);
            left = Spanned::new(
                Expr::Ternary {
                    condition: Box::new(condition),
                    then_expr: Box::new(left),
                    else_expr: Box::new(else_expr),
                },
                span,
            );
            continue;
        }

        // Java ternary: `cond ? then : else`
        if kind == TokenKind::Question && min_prec <= Prec::Ternary {
            p.advance(); // ?
            let then_expr = parse_expr(p, Prec::Assignment);
            p.expect(TokenKind::Colon);
            let else_expr = parse_expr(p, Prec::Ternary);
            let span = left.span.merge(else_expr.span);
            left = Spanned::new(
                Expr::Ternary {
                    condition: Box::new(left),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                span,
            );
            continue;
        }

        // Postfix: call, index, attribute
        if min_prec <= Prec::Postfix {
            match kind {
                TokenKind::LParen => {
                    left = parse_call(p, left);
                    continue;
                }
                TokenKind::LBracket => {
                    left = parse_index(p, left);
                    continue;
                }
                TokenKind::Dot => {
                    p.advance();
                    let name_span = p.expect(TokenKind::Identifier);
                    let name_text =
                        p.source[name_span.start as usize..name_span.end as usize].to_string();
                    let span = left.span.merge(name_span);
                    left = Spanned::new(
                        Expr::Attribute(Box::new(left), Spanned::new(name_text, name_span)),
                        span,
                    );
                    continue;
                }
                TokenKind::QuestionDot => {
                    p.advance();
                    let name_span = p.expect(TokenKind::Identifier);
                    let name_text =
                        p.source[name_span.start as usize..name_span.end as usize].to_string();
                    let span = left.span.merge(name_span);
                    left = Spanned::new(
                        Expr::Attribute(Box::new(left), Spanned::new(name_text, name_span)),
                        span,
                    );
                    continue;
                }
                _ => {}
            }
        }

        // Standard infix binary operators
        if let Some((prec, op)) = infix_prec(kind) {
            if prec < min_prec {
                break;
            }
            p.advance();
            // Left-associative: parse right side at prec+1
            let next_prec = match prec {
                Prec::MulDiv => Prec::Unary,
                Prec::AddSub => Prec::MulDiv,
                Prec::Shift => Prec::AddSub,
                Prec::BitAnd => Prec::Shift,
                Prec::BitXor => Prec::BitAnd,
                Prec::BitOr => Prec::BitXor,
                Prec::Comparison => Prec::BitOr,
                Prec::And => Prec::Not,
                Prec::Or => Prec::And,
                _ => Prec::Or,
            };
            let right = parse_expr(p, next_prec);
            let span = left.span.merge(right.span);
            left = Spanned::new(Expr::BinaryOp(Box::new(left), op, Box::new(right)), span);
            continue;
        }

        break;
    }

    left
}

fn parse_prefix(p: &mut Parser<'_>) -> Spanned<Expr> {
    let kind = p.peek_kind();
    match kind {
        // Unary operators
        TokenKind::Minus => {
            let tok = p.advance();
            let start = tok.span;
            let operand = parse_expr(p, Prec::Unary);
            let span = start.merge(operand.span);
            Spanned::new(Expr::UnaryOp(UnaryOp::Neg, Box::new(operand)), span)
        }
        TokenKind::Plus => {
            let tok = p.advance();
            let start = tok.span;
            let operand = parse_expr(p, Prec::Unary);
            let span = start.merge(operand.span);
            Spanned::new(Expr::UnaryOp(UnaryOp::Pos, Box::new(operand)), span)
        }
        TokenKind::Tilde => {
            let tok = p.advance();
            let start = tok.span;
            let operand = parse_expr(p, Prec::Unary);
            let span = start.merge(operand.span);
            Spanned::new(Expr::UnaryOp(UnaryOp::BitNot, Box::new(operand)), span)
        }
        TokenKind::KwNot => {
            let tok = p.advance();
            let start = tok.span;
            let operand = parse_expr(p, Prec::Not);
            let span = start.merge(operand.span);
            Spanned::new(Expr::UnaryOp(UnaryOp::Not, Box::new(operand)), span)
        }
        TokenKind::Bang => {
            let tok = p.advance();
            let start = tok.span;
            let operand = parse_expr(p, Prec::Unary);
            let span = start.merge(operand.span);
            Spanned::new(Expr::UnaryOp(UnaryOp::LogicalNot, Box::new(operand)), span)
        }
        TokenKind::KwAwait => {
            let tok = p.advance();
            let start = tok.span;
            let operand = parse_expr(p, Prec::Unary);
            let span = start.merge(operand.span);
            Spanned::new(Expr::Await(Box::new(operand)), span)
        }
        TokenKind::KwNew => parse_new_expr(p),
        TokenKind::KwLambda => parse_lambda(p),
        _ => parse_primary(p),
    }
}

fn parse_primary(p: &mut Parser<'_>) -> Spanned<Expr> {
    let kind = p.peek_kind();
    match kind {
        TokenKind::IntLiteral => {
            let tok = p.advance();
            let span = tok.span;
            let text = &p.source[span.start as usize..span.end as usize];
            let value: i128 = parse_int_literal(text);
            Spanned::new(Expr::IntLit(value), span)
        }
        TokenKind::FloatLiteral => {
            let tok = p.advance();
            let span = tok.span;
            let text = &p.source[span.start as usize..span.end as usize];
            let value: f64 = text.parse().unwrap_or(0.0);
            Spanned::new(Expr::FloatLit(value), span)
        }
        TokenKind::StringLiteral | TokenKind::TripleStringLiteral => {
            let tok = p.advance();
            let span = tok.span;
            let raw = &p.source[span.start as usize..span.end as usize];
            let is_fstring = raw.starts_with('f') || raw.starts_with('F');
            let value = strip_string_quotes(raw);
            if is_fstring {
                parse_fstring_parts(&value, span)
            } else {
                Spanned::new(Expr::StringLit(value), span)
            }
        }
        TokenKind::BoolTrue => {
            let tok = p.advance();
            Spanned::new(Expr::BoolLit(true), tok.span)
        }
        TokenKind::BoolFalse => {
            let tok = p.advance();
            Spanned::new(Expr::BoolLit(false), tok.span)
        }
        TokenKind::NullNone | TokenKind::NullNull => {
            let tok = p.advance();
            Spanned::new(Expr::NullLit, tok.span)
        }
        TokenKind::Identifier
        | TokenKind::KwThis
        | TokenKind::KwSuper
        // Contextual keywords used as identifiers (e.g. `val = 5` assignment)
        | TokenKind::KwVal
        | TokenKind::KwVar
        | TokenKind::KwConst => {
            let tok = p.advance();
            let span = tok.span;
            let name = p.source[span.start as usize..span.end as usize].to_string();
            Spanned::new(Expr::Ident(name), span)
        }
        TokenKind::LParen => parse_paren_expr(p),
        TokenKind::LBracket => parse_list_literal(p),
        TokenKind::LBrace => parse_dict_or_set_literal(p),
        _ => p.error_expr(&format!("expected expression, found {}", p.peek_kind())),
    }
}

fn parse_int_literal(text: &str) -> i128 {
    let text = text.replace('_', "");
    if text.starts_with("0x") || text.starts_with("0X") {
        i128::from_str_radix(&text[2..], 16).unwrap_or(0)
    } else if text.starts_with("0o") || text.starts_with("0O") {
        i128::from_str_radix(&text[2..], 8).unwrap_or(0)
    } else if text.starts_with("0b") || text.starts_with("0B") {
        i128::from_str_radix(&text[2..], 2).unwrap_or(0)
    } else {
        text.parse().unwrap_or(0)
    }
}

/// Build an expression AST node for the raw text inside `{...}` in an f-string.
///
/// Handles:
/// - Simple identifiers: `name` → `Ident("name")`
/// - Dotted attribute paths: `this.width` → `Attribute(Ident("this"), "width")`
/// - Deeper paths: `a.b.c` → `Attribute(Attribute(Ident("a"), "b"), "c")`
///
/// Everything else (calls, index access, arithmetic) falls back to a single `Ident`
/// containing the raw text, which will be flagged by the semantic analyser if invalid.
fn build_fstring_expr(text: &str, span: Span) -> Spanned<Expr> {
    // Fast path: no dot — plain identifier.
    if !text.contains('.') {
        return Spanned::new(Expr::Ident(text.to_string()), span);
    }
    // Dotted path: split on `.` and build left-to-right Attribute chain.
    let parts: Vec<&str> = text.splitn(2, '.').collect();
    let head = Spanned::new(Expr::Ident(parts[0].trim().to_string()), span);
    if parts.len() == 1 || parts[1].trim().is_empty() {
        return head;
    }
    // Recursively handle remaining parts.
    let tail = build_fstring_expr(parts[1].trim(), span);
    // The tail might be a plain Ident or a deeper Attribute — either way, wrap head around it.
    match tail.node {
        Expr::Ident(field) => {
            Spanned::new(Expr::Attribute(Box::new(head), Spanned::new(field, span)), span)
        }
        Expr::Attribute(inner_obj, field) => {
            // Re-assemble: head.inner_obj.field  →  Attribute(Attribute(head, first_of_tail), rest)
            // Walk the chain to prepend `head`.
            let first_attr = if let Expr::Ident(ref name) = inner_obj.node {
                Spanned::new(
                    Expr::Attribute(Box::new(head), Spanned::new(name.clone(), span)),
                    span,
                )
            } else {
                head // fallback
            };
            Spanned::new(Expr::Attribute(Box::new(first_attr), field), span)
        }
        _ => head,
    }
}

/// Expand f-string content like `Hello, {name}!` into a concatenation AST:
///   "Hello, " + str(name) + "!"
fn parse_fstring_parts(content: &str, span: Span) -> Spanned<Expr> {
    let mut parts: Vec<Spanned<Expr>> = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                buf.push('{');
                i += 2;
                continue;
            }
            if !buf.is_empty() {
                parts.push(Spanned::new(Expr::StringLit(buf.clone()), span));
                buf.clear();
            }
            i += 1; // skip '{'
            let mut expr_str = String::new();
            let mut depth = 1u32;
            while i < chars.len() && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                expr_str.push(chars[i]);
                i += 1;
            }
            i += 1; // skip closing '}'
            let trimmed = expr_str.trim();
            if !trimmed.is_empty() {
                // Build the inner expression.
                // Support dotted paths like `this.width` or `obj.field.sub`.
                // More complex expressions (calls, indexing, arithmetic) remain as
                // a single Ident for now — a future parser-based approach can extend this.
                let inner_expr = build_fstring_expr(trimmed, span);
                let str_callee = Spanned::new(Expr::Ident("str".to_string()), span);
                let call = Expr::Call(
                    Box::new(str_callee),
                    vec![Argument {
                        name: None,
                        value: inner_expr,
                    }],
                );
                parts.push(Spanned::new(call, span));
            }
        } else if chars[i] == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
            buf.push('}');
            i += 2;
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }

    if !buf.is_empty() {
        parts.push(Spanned::new(Expr::StringLit(buf), span));
    }

    if parts.is_empty() {
        return Spanned::new(Expr::StringLit(String::new()), span);
    }

    let mut result = parts.remove(0);
    for part in parts {
        result = Spanned::new(
            Expr::BinaryOp(Box::new(result), BinOp::Add, Box::new(part)),
            span,
        );
    }
    result
}

fn strip_string_quotes(raw: &str) -> String {
    // Handle prefixed strings (f"...", r"...", b"...")
    let s = if raw.starts_with('f')
        || raw.starts_with('r')
        || raw.starts_with('b')
        || raw.starts_with('F')
        || raw.starts_with('R')
        || raw.starts_with('B')
    {
        &raw[1..]
    } else {
        raw
    };

    if s.starts_with("\"\"\"") || s.starts_with("'''") {
        let inner = &s[3..];
        if inner.len() >= 3 {
            inner[..inner.len() - 3].to_string()
        } else {
            inner.to_string()
        }
    } else if s.starts_with('"') || s.starts_with('\'') {
        let inner = &s[1..];
        if inner.ends_with(s.chars().next().unwrap()) && !inner.is_empty() {
            inner[..inner.len() - 1].to_string()
        } else {
            inner.to_string()
        }
    } else {
        s.to_string()
    }
}

fn parse_paren_expr(p: &mut Parser<'_>) -> Spanned<Expr> {
    let start = p.advance().span; // (
    if p.at(TokenKind::RParen) {
        // Empty tuple / unit — treat as empty list for now
        let end = p.advance().span;
        return Spanned::new(Expr::List(Vec::new()), start.merge(end));
    }
    let expr = parse_expr(p, Prec::None);
    let end = p.expect(TokenKind::RParen);
    // Re-wrap with the full span including parens
    Spanned::new(expr.node, start.merge(end))
}

fn parse_call(p: &mut Parser<'_>, callee: Spanned<Expr>) -> Spanned<Expr> {
    p.advance(); // (
    let mut args = Vec::new();
    while !p.at(TokenKind::RParen) && !p.at_eof() {
        p.skip_terminators();
        if p.at(TokenKind::RParen) {
            break;
        }

        // Check for keyword argument: name=value
        let arg = if p.peek_kind() == TokenKind::Identifier && p.peek_nth(1) == TokenKind::Eq {
            let name_tok = p.advance();
            let name_span = name_tok.span;
            let name = p.source[name_span.start as usize..name_span.end as usize].to_string();
            p.advance(); // =
            let value = parse_expr(p, Prec::None);
            Argument {
                name: Some(Spanned::new(name, name_span)),
                value,
            }
        } else {
            let value = parse_expr(p, Prec::None);
            Argument { name: None, value }
        };
        args.push(arg);

        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    p.skip_terminators();
    let end = p.expect(TokenKind::RParen);
    let span = callee.span.merge(end);
    Spanned::new(Expr::Call(Box::new(callee), args), span)
}

fn parse_index(p: &mut Parser<'_>, obj: Spanned<Expr>) -> Spanned<Expr> {
    p.advance(); // [
    let index = parse_expr(p, Prec::None);
    let end = p.expect(TokenKind::RBracket);
    let span = obj.span.merge(end);
    Spanned::new(Expr::Index(Box::new(obj), Box::new(index)), span)
}

fn parse_list_literal(p: &mut Parser<'_>) -> Spanned<Expr> {
    let start = p.advance().span; // [
    let mut elements = Vec::new();
    while !p.at(TokenKind::RBracket) && !p.at_eof() {
        p.skip_terminators();
        if p.at(TokenKind::RBracket) {
            break;
        }
        let elem = parse_expr(p, Prec::None);

        // Check for list comprehension: [expr for x in iter]
        if elements.is_empty() && p.at(TokenKind::KwFor) {
            return parse_list_comp(p, elem, start);
        }

        elements.push(elem);
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    let end = p.expect(TokenKind::RBracket);
    Spanned::new(Expr::List(elements), start.merge(end))
}

fn parse_list_comp(p: &mut Parser<'_>, element: Spanned<Expr>, start: Span) -> Spanned<Expr> {
    let mut clauses = Vec::new();
    while p.eat(TokenKind::KwFor) {
        // Use Prec::BitOr so that `in` (Prec::Comparison) is NOT consumed as part of
        // the target expression — it must remain as a separator keyword.
        let target = parse_expr(p, Prec::BitOr);
        p.expect(TokenKind::KwIn);
        // Parse iter up to `if`, `for`, or `]` — stop at Comparison level so
        // chained comprehensions and conditionals work correctly.
        let iter = parse_expr(p, Prec::BitOr);
        let mut conditions = Vec::new();
        while p.eat(TokenKind::KwIf) {
            let cond = parse_expr(p, Prec::Comparison);
            conditions.push(cond);
        }
        clauses.push(CompClause {
            target,
            iter,
            conditions,
        });
    }
    let end = p.expect(TokenKind::RBracket);
    Spanned::new(
        Expr::ListComp {
            element: Box::new(element),
            clauses,
        },
        start.merge(end),
    )
}

fn parse_dict_or_set_literal(p: &mut Parser<'_>) -> Spanned<Expr> {
    let start = p.advance().span; // {

    p.skip_terminators(); // allow `{\n` opening
    if p.at(TokenKind::RBrace) {
        let end = p.advance().span;
        return Spanned::new(Expr::Dict(Vec::new()), start.merge(end));
    }

    let first = parse_expr(p, Prec::None);

    // Dict: {key: value, ...}
    if p.at(TokenKind::Colon) {
        p.advance();
        let val = parse_expr(p, Prec::None);
        let mut pairs = vec![(first, val)];
        loop {
            p.skip_terminators(); // skip newline after value (before comma or })
            if !p.eat(TokenKind::Comma) {
                break;
            }
            p.skip_terminators();
            if p.at(TokenKind::RBrace) {
                break;
            }
            let k = parse_expr(p, Prec::None);
            p.expect(TokenKind::Colon);
            let v = parse_expr(p, Prec::None);
            pairs.push((k, v));
        }
        p.skip_terminators();
        let end = p.expect(TokenKind::RBrace);
        return Spanned::new(Expr::Dict(pairs), start.merge(end));
    }

    // Set: {a, b, ...}
    let mut elements = vec![first];
    loop {
        p.skip_terminators();
        if !p.eat(TokenKind::Comma) {
            break;
        }
        p.skip_terminators();
        if p.at(TokenKind::RBrace) {
            break;
        }
        elements.push(parse_expr(p, Prec::None));
    }
    p.skip_terminators();
    let end = p.expect(TokenKind::RBrace);
    Spanned::new(Expr::Set(elements), start.merge(end))
}

fn parse_new_expr(p: &mut Parser<'_>) -> Spanned<Expr> {
    let start = p.advance().span; // new
    let type_expr = crate::types::parse_type_expr(p);
    let mut args = Vec::new();
    if p.at(TokenKind::LParen) {
        p.advance(); // (
        while !p.at(TokenKind::RParen) && !p.at_eof() {
            p.skip_terminators();
            if p.at(TokenKind::RParen) {
                break;
            }
            let value = parse_expr(p, Prec::None);
            args.push(Argument { name: None, value });
            if !p.eat(TokenKind::Comma) {
                break;
            }
        }
        let end = p.expect(TokenKind::RParen);
        let span = start.merge(end);
        Spanned::new(Expr::New(type_expr, args), span)
    } else {
        let span = start.merge(type_expr.span);
        Spanned::new(Expr::New(type_expr, args), span)
    }
}

fn parse_lambda(p: &mut Parser<'_>) -> Spanned<Expr> {
    let start = p.advance().span; // lambda
    let mut params = Vec::new();
    while !p.at(TokenKind::Colon) && !p.at_eof() {
        let name_span = p.expect(TokenKind::Identifier);
        let name = p.source[name_span.start as usize..name_span.end as usize].to_string();
        params.push(Param {
            name: Spanned::new(name, name_span),
            type_ann: None,
            default: None,
            is_vararg: false,
            is_kwarg: false,
        });
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::Colon);
    let body = parse_expr(p, Prec::Ternary);
    let span = start.merge(body.span);
    Spanned::new(Expr::Lambda(params, Box::new(body)), span)
}

/// Public convenience: parse expression from parser.
pub(crate) fn parse_expression(p: &mut Parser<'_>) -> Spanned<Expr> {
    parse_expr(p, Prec::None)
}
