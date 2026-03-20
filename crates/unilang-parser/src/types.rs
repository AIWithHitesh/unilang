// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Type expression parser for UniLang.

use unilang_common::span::Spanned;
use unilang_lexer::token::TokenKind;

use crate::ast::TypeExpr;
use crate::parser::Parser;

/// Parse a type expression: `int`, `String`, `List<T>`, `Optional<int>`, `int[]`, `a.b.C`
pub(crate) fn parse_type_expr(p: &mut Parser<'_>) -> Spanned<TypeExpr> {
    let mut ty = parse_primary_type(p);

    // Handle array suffix: `int[]`
    while p.at(TokenKind::LBracket) && p.peek_nth(1) == TokenKind::RBracket {
        p.advance(); // [
        let end = p.advance().span; // ]
        let span = ty.span.merge(end);
        ty = Spanned::new(TypeExpr::Array(Box::new(ty)), span);
    }

    // Handle union: `int | str`
    if p.at(TokenKind::Pipe) {
        let mut types = vec![ty];
        while p.eat(TokenKind::Pipe) {
            let next = parse_primary_type(p);
            // Handle array suffix after union member
            types.push(next);
        }
        let start = types.first().unwrap().span;
        let end = types.last().unwrap().span;
        let span = start.merge(end);
        ty = Spanned::new(TypeExpr::Union(types), span);
    }

    ty
}

fn parse_primary_type(p: &mut Parser<'_>) -> Spanned<TypeExpr> {
    let kind = p.peek_kind();

    match kind {
        TokenKind::Identifier | TokenKind::KwVoid => {
            let tok = p.advance();
            let span = tok.span;
            let name = p.source[span.start as usize..span.end as usize].to_string();

            // Check for qualified name: a.b.C
            let mut parts = vec![name.clone()];
            while p.at(TokenKind::Dot) && p.peek_nth(1) == TokenKind::Identifier {
                p.advance(); // .
                let next_tok = p.advance();
                let next_span = next_tok.span;
                parts.push(p.source[next_span.start as usize..next_span.end as usize].to_string());
            }

            // Check for generic params: Name<T, U>
            if p.at(TokenKind::Lt) {
                let type_params = parse_generic_args(p);
                let end_span = type_params.last().map(|t| t.span).unwrap_or(span);
                let full_span = span.merge(end_span);

                let base = if parts.len() == 1 {
                    Spanned::new(TypeExpr::Named(parts.remove(0)), span)
                } else {
                    Spanned::new(TypeExpr::Qualified(parts), span)
                };

                // Merge span to include closing >
                Spanned::new(TypeExpr::Generic(Box::new(base), type_params), full_span)
            } else if parts.len() == 1 {
                Spanned::new(TypeExpr::Named(parts.remove(0)), span)
            } else {
                let last_span = p.tokens_span_for_parts(&parts, span);
                Spanned::new(TypeExpr::Qualified(parts), last_span)
            }
        }
        TokenKind::Question => {
            // Optional shorthand: ?Type
            let start = p.advance().span;
            let inner = parse_primary_type(p);
            let span = start.merge(inner.span);
            Spanned::new(TypeExpr::Optional(Box::new(inner)), span)
        }
        TokenKind::LParen => {
            // Tuple type: (int, str)
            let start = p.advance().span;
            let mut types = Vec::new();
            while !p.at(TokenKind::RParen) && !p.at_eof() {
                types.push(parse_type_expr(p));
                if !p.eat(TokenKind::Comma) {
                    break;
                }
            }
            let end = p.expect(TokenKind::RParen);
            Spanned::new(TypeExpr::Tuple(types), start.merge(end))
        }
        _ => {
            // Inferred or error
            let span = p.current_span();
            Spanned::new(TypeExpr::Inferred, span)
        }
    }
}

fn parse_generic_args(p: &mut Parser<'_>) -> Vec<Spanned<TypeExpr>> {
    let mut args = Vec::new();
    if !p.eat(TokenKind::Lt) {
        return args;
    }
    loop {
        if p.at(TokenKind::Gt) || p.at_eof() {
            break;
        }
        args.push(parse_type_expr(p));
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::Gt);
    args
}

/// Helper on Parser to compute span for qualified name parts.
/// This is a simple approach — just use the original span extended.
impl Parser<'_> {
    pub(crate) fn tokens_span_for_parts(
        &self,
        _parts: &[String],
        base_span: unilang_common::span::Span,
    ) -> unilang_common::span::Span {
        // We've already advanced past the dots and names, so look backwards
        // to find the actual end position. Use a simple heuristic:
        // the current position minus 1 token should give us the end.
        if self.pos > 0 {
            let last = &self.tokens[self.pos - 1];
            base_span.merge(last.span)
        } else {
            base_span
        }
    }
}
