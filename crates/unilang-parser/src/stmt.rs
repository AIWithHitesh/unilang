// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Statement parser for UniLang.

use unilang_common::span::Spanned;
use unilang_common::syntax_origin::SyntaxOrigin;
use unilang_lexer::token::TokenKind;

use crate::ast::*;
use crate::expr;
use crate::parser::Parser;

/// Parse a single statement.
pub(crate) fn parse_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    // Parse decorators first
    let decorators = parse_decorators(p);

    // Parse visibility modifiers and other modifiers
    let (visibility, modifiers, syntax_hint) = parse_modifiers(p);

    let kind = p.peek_kind();

    match kind {
        TokenKind::KwDef | TokenKind::KwAsync => {
            parse_function_decl(p, visibility, modifiers, decorators, false)
        }
        TokenKind::KwClass => parse_class_decl(p, visibility, modifiers, decorators),
        TokenKind::KwIf => parse_if_stmt(p),
        TokenKind::KwFor => parse_for_stmt(p),
        TokenKind::KwWhile => parse_while_stmt(p),
        TokenKind::KwDo => parse_do_while(p),
        TokenKind::KwTry => parse_try_stmt(p),
        TokenKind::KwWith => parse_with_stmt(p),
        TokenKind::KwReturn => parse_return(p),
        TokenKind::KwRaise => parse_throw(p, SyntaxOrigin::Python),
        TokenKind::KwThrow => parse_throw(p, SyntaxOrigin::Java),
        TokenKind::KwBreak => {
            let tok = p.advance();
            Spanned::new(Stmt::Break, tok.span)
        }
        TokenKind::KwContinue => {
            let tok = p.advance();
            Spanned::new(Stmt::Continue, tok.span)
        }
        TokenKind::KwPass => {
            let tok = p.advance();
            Spanned::new(Stmt::Pass, tok.span)
        }
        TokenKind::KwAssert => parse_assert(p),
        TokenKind::KwImport => parse_import_stmt(p),
        TokenKind::KwFrom => parse_from_import(p),
        TokenKind::KwVar | TokenKind::KwVal | TokenKind::KwConst
            // Only treat as a declaration keyword when followed by an identifier name.
            // If followed by `=`, `+=`, etc. it's being used as an identifier (e.g. `val = 5`).
            if matches!(p.peek_nth(1), TokenKind::Identifier | TokenKind::Colon) =>
        {
            parse_var_decl_keyword(p, modifiers)
        }
        TokenKind::Identifier | TokenKind::KwVoid
            if has_modifiers_or_visibility(visibility, &modifiers) =>
        {
            // Could be Java-style: `int x = ...` or `int foo(...)` with modifiers
            // KwVoid handles `public void method()` — void is a keyword, not an Identifier
            parse_java_decl_or_expr(p, visibility, modifiers, decorators, syntax_hint)
        }
        TokenKind::Identifier => {
            // Could be Java-style typed declaration: `int x = 5` or `String foo(...)`
            // or just an expression statement
            parse_expr_or_java_decl(p)
        }
        _ => {
            if !decorators.is_empty() || has_modifiers_or_visibility(visibility, &modifiers) {
                // We parsed modifiers but didn't find a declaration
                p.error_stmt(&format!(
                    "expected declaration after modifiers, found {}",
                    p.peek_kind()
                ))
            } else {
                // Expression statement
                let expr = expr::parse_expression(p);
                let span = expr.span;
                Spanned::new(Stmt::Expr(expr.node), span)
            }
        }
    }
}

fn has_modifiers_or_visibility(vis: Visibility, mods: &[Modifier]) -> bool {
    vis != Visibility::Default || !mods.is_empty()
}

/// Parse @decorator annotations.
fn parse_decorators(p: &mut Parser<'_>) -> Vec<Spanned<Decorator>> {
    let mut decorators = Vec::new();
    while p.at(TokenKind::At) {
        let start = p.advance().span; // @
        let name_span = p.expect(TokenKind::Identifier);
        let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

        let mut args = Vec::new();
        let end = if p.at(TokenKind::LParen) {
            p.advance(); // (
            while !p.at(TokenKind::RParen) && !p.at_eof() {
                args.push(expr::parse_expression(p));
                if !p.eat(TokenKind::Comma) {
                    break;
                }
            }
            p.expect(TokenKind::RParen)
        } else {
            name_span
        };
        decorators.push(Spanned::new(
            Decorator {
                name: Spanned::new(name, name_span),
                args,
            },
            start.merge(end),
        ));
        p.skip_terminators();
    }
    decorators
}

/// Parse visibility and modifier keywords.
fn parse_modifiers(p: &mut Parser<'_>) -> (Visibility, Vec<Modifier>, SyntaxOrigin) {
    let mut visibility = Visibility::Default;
    let mut modifiers = Vec::new();
    let mut syntax = SyntaxOrigin::Ambiguous;

    loop {
        match p.peek_kind() {
            TokenKind::KwPublic => {
                p.advance();
                visibility = Visibility::Public;
                syntax = SyntaxOrigin::Java;
            }
            TokenKind::KwPrivate => {
                p.advance();
                visibility = Visibility::Private;
                syntax = SyntaxOrigin::Java;
            }
            TokenKind::KwProtected => {
                p.advance();
                visibility = Visibility::Protected;
                syntax = SyntaxOrigin::Java;
            }
            TokenKind::KwStatic => {
                p.advance();
                modifiers.push(Modifier::Static);
                syntax = SyntaxOrigin::Java;
            }
            TokenKind::KwFinal => {
                p.advance();
                modifiers.push(Modifier::Final);
                syntax = SyntaxOrigin::Java;
            }
            TokenKind::KwAbstract => {
                p.advance();
                modifiers.push(Modifier::Abstract);
                syntax = SyntaxOrigin::Java;
            }
            TokenKind::KwNative => {
                p.advance();
                modifiers.push(Modifier::Native);
            }
            TokenKind::KwSynchronized => {
                p.advance();
                modifiers.push(Modifier::Synchronized);
            }
            TokenKind::KwVolatile => {
                p.advance();
                modifiers.push(Modifier::Volatile);
            }
            TokenKind::KwTransient => {
                p.advance();
                modifiers.push(Modifier::Transient);
            }
            _ => break,
        }
    }

    (visibility, modifiers, syntax)
}

// ── Variable Declarations ────────────────────────────────

fn parse_var_decl_keyword(p: &mut Parser<'_>, modifiers: Vec<Modifier>) -> Spanned<Stmt> {
    let start = p.advance().span; // var/val/const
    let kw_text = &p.source[start.start as usize..start.end as usize];
    let syntax = match kw_text {
        "val" | "const" => SyntaxOrigin::UniLang,
        "var" => SyntaxOrigin::UniLang,
        _ => SyntaxOrigin::Ambiguous,
    };

    let name_span = p.expect(TokenKind::Identifier);
    let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

    // Optional type annotation: var x: int = ...
    let type_ann = if p.eat(TokenKind::Colon) {
        Some(crate::types::parse_type_expr(p))
    } else {
        None
    };

    let initializer = if p.eat(TokenKind::Eq) {
        Some(expr::parse_expression(p))
    } else {
        None
    };

    let end = initializer
        .as_ref()
        .map(|e| e.span)
        .or(type_ann.as_ref().map(|t| t.span))
        .unwrap_or(name_span);

    Spanned::new(
        Stmt::VarDecl(VarDecl {
            name: Spanned::new(name, name_span),
            type_ann,
            initializer,
            modifiers,
            syntax,
        }),
        start.merge(end),
    )
}

// ── Function Declaration ─────────────────────────────────

fn parse_function_decl(
    p: &mut Parser<'_>,
    visibility: Visibility,
    modifiers: Vec<Modifier>,
    decorators: Vec<Spanned<Decorator>>,
    _is_java: bool,
) -> Spanned<Stmt> {
    let is_async = if p.at(TokenKind::KwAsync) {
        p.advance();
        true
    } else {
        false
    };

    let start = p.advance().span; // def

    let name_span = p.expect(TokenKind::Identifier);
    let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

    p.expect(TokenKind::LParen);
    let params = parse_param_list(p);
    p.expect(TokenKind::RParen);

    // Optional return type: -> Type or : Type (after params)
    let return_type = if p.eat(TokenKind::Arrow) {
        Some(crate::types::parse_type_expr(p))
    } else {
        None
    };

    let body = parse_block(p);
    let end_span = body.statements.last().map(|s| s.span).unwrap_or(name_span);

    Spanned::new(
        Stmt::FunctionDecl(FunctionDecl {
            name: Spanned::new(name, name_span),
            params,
            return_type,
            body,
            visibility,
            modifiers,
            decorators,
            is_async,
            syntax: SyntaxOrigin::Python,
        }),
        start.merge(end_span),
    )
}

/// Parse a Java-style function/method or variable declaration when we've
/// already consumed modifiers and the current token is an identifier (the type).
fn parse_java_decl_or_expr(
    p: &mut Parser<'_>,
    visibility: Visibility,
    modifiers: Vec<Modifier>,
    decorators: Vec<Spanned<Decorator>>,
    _syntax_hint: SyntaxOrigin,
) -> Spanned<Stmt> {
    // Current token is the type name (e.g., `int`, `String`, `void`)
    // We need: type name ( params ) { body }  — method
    // or: type name = expr ;  — variable
    // or: class Name { ... }  — class

    if p.at(TokenKind::KwClass) {
        return parse_class_decl(p, visibility, modifiers, decorators);
    }

    if p.at(TokenKind::KwDef) || p.at(TokenKind::KwAsync) {
        return parse_function_decl(p, visibility, modifiers, decorators, false);
    }

    let return_type = crate::types::parse_type_expr(p);
    let start = return_type.span;

    if !p.at(TokenKind::Identifier) {
        // Not a valid Java decl — emit error
        return p.error_stmt(&format!(
            "expected identifier after type, found {}",
            p.peek_kind()
        ));
    }

    let name_span = p.expect(TokenKind::Identifier);
    let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

    // Method: type name(params) { ... }
    if p.at(TokenKind::LParen) {
        p.advance(); // (
        let params = parse_typed_param_list(p);
        p.expect(TokenKind::RParen);

        let body = parse_block(p);
        let end_span = body.statements.last().map(|s| s.span).unwrap_or(name_span);

        return Spanned::new(
            Stmt::FunctionDecl(FunctionDecl {
                name: Spanned::new(name, name_span),
                params,
                return_type: Some(return_type),
                body,
                visibility,
                modifiers,
                decorators,
                is_async: false,
                syntax: SyntaxOrigin::Java,
            }),
            start.merge(end_span),
        );
    }

    // Variable: type name = expr
    let initializer = if p.eat(TokenKind::Eq) {
        Some(expr::parse_expression(p))
    } else {
        None
    };

    let end = initializer.as_ref().map(|e| e.span).unwrap_or(name_span);

    Spanned::new(
        Stmt::VarDecl(VarDecl {
            name: Spanned::new(name, name_span),
            type_ann: Some(return_type),
            initializer,
            modifiers,
            syntax: SyntaxOrigin::Java,
        }),
        start.merge(end),
    )
}

/// Try to parse an expression — but first check if this looks like a Java-style
/// typed declaration (e.g., `int x = 5` or `String name`).
fn parse_expr_or_java_decl(p: &mut Parser<'_>) -> Spanned<Stmt> {
    // Lookahead: identifier followed by identifier = Java typed decl
    // e.g., `int x`, `String name = "foo"`, `int add(int a, int b) { ... }`
    if p.peek_kind() == TokenKind::Identifier && p.peek_nth(1) == TokenKind::Identifier {
        // Check if this is `type name(` — a method, or `type name =` / `type name;` / `type name\n` — a var
        let next2 = p.peek_nth(2);
        if next2 == TokenKind::LParen
            || next2 == TokenKind::Eq
            || next2 == TokenKind::Semicolon
            || next2 == TokenKind::Newline
            || next2 == TokenKind::Eof
        {
            return parse_java_decl_or_expr(
                p,
                Visibility::Default,
                Vec::new(),
                Vec::new(),
                SyntaxOrigin::Java,
            );
        }
    }

    // Regular expression statement
    let expr = expr::parse_expression(p);
    let span = expr.span;
    Spanned::new(Stmt::Expr(expr.node), span)
}

// ── Parameter Lists ──────────────────────────────────────

fn parse_param_list(p: &mut Parser<'_>) -> Vec<Param> {
    let mut params = Vec::new();
    while !p.at(TokenKind::RParen) && !p.at_eof() {
        p.skip_terminators();
        if p.at(TokenKind::RParen) {
            break;
        }

        let is_vararg = p.eat(TokenKind::Star);
        let is_kwarg = if !is_vararg {
            p.eat(TokenKind::DoubleStar)
        } else {
            false
        };

        let name_span = p.expect(TokenKind::Identifier);
        let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

        // Optional type annotation
        let type_ann = if p.eat(TokenKind::Colon) {
            Some(crate::types::parse_type_expr(p))
        } else {
            None
        };

        // Optional default value
        let default = if p.eat(TokenKind::Eq) {
            Some(expr::parse_expression(p))
        } else {
            None
        };

        params.push(Param {
            name: Spanned::new(name, name_span),
            type_ann,
            default,
            is_vararg,
            is_kwarg,
        });

        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    params
}

/// Parse Java-style typed parameters: `int a, String b`
fn parse_typed_param_list(p: &mut Parser<'_>) -> Vec<Param> {
    let mut params = Vec::new();
    while !p.at(TokenKind::RParen) && !p.at_eof() {
        p.skip_terminators();
        if p.at(TokenKind::RParen) {
            break;
        }

        // type name
        let type_ann = crate::types::parse_type_expr(p);

        if !p.at(TokenKind::Identifier) {
            // Maybe it's a Python-style param without type
            // Use the type as the name
            let name = match &type_ann.node {
                TypeExpr::Named(n) => n.clone(),
                _ => "?".to_string(),
            };
            params.push(Param {
                name: Spanned::new(name, type_ann.span),
                type_ann: None,
                default: None,
                is_vararg: false,
                is_kwarg: false,
            });
            if !p.eat(TokenKind::Comma) {
                break;
            }
            continue;
        }

        let name_span = p.expect(TokenKind::Identifier);
        let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

        params.push(Param {
            name: Spanned::new(name, name_span),
            type_ann: Some(type_ann),
            default: None,
            is_vararg: false,
            is_kwarg: false,
        });

        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    params
}

// ── Class Declaration ────────────────────────────────────

fn parse_class_decl(
    p: &mut Parser<'_>,
    visibility: Visibility,
    modifiers: Vec<Modifier>,
    decorators: Vec<Spanned<Decorator>>,
) -> Spanned<Stmt> {
    let start = p.advance().span; // class

    let name_span = p.expect(TokenKind::Identifier);
    let name = p.source[name_span.start as usize..name_span.end as usize].to_string();

    // Type params: class Foo<T>
    let type_params = if p.at(TokenKind::Lt) {
        parse_type_param_list(p)
    } else {
        Vec::new()
    };

    // Python-style bases: class Foo(Base1, Base2):
    let mut bases = Vec::new();
    if p.at(TokenKind::LParen) {
        p.advance();
        while !p.at(TokenKind::RParen) && !p.at_eof() {
            bases.push(crate::types::parse_type_expr(p));
            if !p.eat(TokenKind::Comma) {
                break;
            }
        }
        p.expect(TokenKind::RParen);
    }

    // Java-style extends/implements
    let extends = if p.eat(TokenKind::KwExtends) {
        Some(crate::types::parse_type_expr(p))
    } else {
        None
    };

    let mut implements = Vec::new();
    if p.eat(TokenKind::KwImplements) {
        loop {
            implements.push(crate::types::parse_type_expr(p));
            if !p.eat(TokenKind::Comma) {
                break;
            }
        }
    }

    // Determine syntax origin based on how block is delimited
    let (body_stmts, syntax) = parse_class_body(p);

    let end = body_stmts.last().map(|s| s.span).unwrap_or(name_span);

    Spanned::new(
        Stmt::ClassDecl(ClassDecl {
            name: Spanned::new(name, name_span),
            type_params,
            bases,
            extends,
            implements,
            body: body_stmts,
            visibility,
            modifiers,
            decorators,
            syntax,
        }),
        start.merge(end),
    )
}

fn parse_type_param_list(p: &mut Parser<'_>) -> Vec<Spanned<TypeExpr>> {
    let mut params = Vec::new();
    p.expect(TokenKind::Lt);
    loop {
        if p.at(TokenKind::Gt) || p.at_eof() {
            break;
        }
        params.push(crate::types::parse_type_expr(p));
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::Gt);
    params
}

fn parse_class_body(p: &mut Parser<'_>) -> (Vec<Spanned<Stmt>>, SyntaxOrigin) {
    let mut stmts = Vec::new();

    if p.at(TokenKind::LBrace) {
        // Java-style
        p.advance(); // {
        p.skip_terminators();
        while !p.at(TokenKind::RBrace) && !p.at_eof() {
            let before = p.pos;
            stmts.push(parse_stmt(p));
            p.skip_terminators();
            if p.pos == before {
                p.advance(); // force progress to prevent infinite loop
            }
        }
        p.expect(TokenKind::RBrace);
        (stmts, SyntaxOrigin::Java)
    } else if p.at(TokenKind::Colon) {
        // Python-style
        p.advance(); // :
        if p.at(TokenKind::Newline) {
            p.advance();
        }
        if p.at(TokenKind::Indent) {
            p.advance();
            while !p.at(TokenKind::Dedent) && !p.at_eof() {
                let before = p.pos;
                stmts.push(parse_stmt(p));
                p.skip_terminators();
                if p.pos == before {
                    p.advance(); // force progress to prevent infinite loop
                }
            }
            p.eat(TokenKind::Dedent);
        }
        (stmts, SyntaxOrigin::Python)
    } else {
        // Empty class or error
        (stmts, SyntaxOrigin::Ambiguous)
    }
}

// ── Block Parsing ────────────────────────────────────────

pub(crate) fn parse_block(p: &mut Parser<'_>) -> Block {
    if p.at(TokenKind::LBrace) {
        parse_brace_block(p)
    } else if p.at(TokenKind::Colon) {
        parse_indent_block(p)
    } else {
        // Error: expected block
        Block {
            style: BlockStyle::Braces,
            statements: Vec::new(),
        }
    }
}

fn parse_brace_block(p: &mut Parser<'_>) -> Block {
    p.advance(); // {
    p.skip_terminators();
    let mut stmts = Vec::new();
    while !p.at(TokenKind::RBrace) && !p.at_eof() {
        let before = p.pos;
        stmts.push(parse_stmt(p));
        p.skip_terminators();
        if p.pos == before {
            p.advance(); // force progress to prevent infinite loop
        }
    }
    p.expect(TokenKind::RBrace);
    Block {
        style: BlockStyle::Braces,
        statements: stmts,
    }
}

fn parse_indent_block(p: &mut Parser<'_>) -> Block {
    p.advance(); // :
                 // Expect newline then indent
    if p.at(TokenKind::Newline) {
        p.advance();
    }

    let mut stmts = Vec::new();
    if p.at(TokenKind::Indent) {
        p.advance();
        while !p.at(TokenKind::Dedent) && !p.at_eof() {
            let before = p.pos;
            stmts.push(parse_stmt(p));
            p.skip_terminators();
            if p.pos == before {
                p.advance(); // force progress to prevent infinite loop
            }
        }
        p.eat(TokenKind::Dedent);
    } else {
        // Single-line block (e.g., `if x: return y`)
        stmts.push(parse_stmt(p));
    }

    Block {
        style: BlockStyle::Indentation,
        statements: stmts,
    }
}

// ── If Statement ─────────────────────────────────────────

fn parse_if_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // if

    // Parse condition — optionally wrapped in parens (Java-style)
    let has_paren = p.eat(TokenKind::LParen);
    let condition = expr::parse_expression(p);
    if has_paren {
        p.expect(TokenKind::RParen);
    }

    let then_block = parse_block(p);

    p.skip_terminators();

    // Parse elif clauses
    let mut elif_clauses = Vec::new();
    while p.at(TokenKind::KwElif) {
        p.advance(); // elif
        let elif_cond = expr::parse_expression(p);
        let elif_body = parse_block(p);
        elif_clauses.push((elif_cond, elif_body));
        p.skip_terminators();
    }

    // Parse else block
    let else_block = if p.at(TokenKind::KwElse) {
        p.advance(); // else
                     // Java style: else if (...) { ... }
        if p.at(TokenKind::KwIf) {
            // Treat `else if` as an elif
            let inner = parse_if_stmt(p);
            Some(Block {
                style: then_block.style,
                statements: vec![inner],
            })
        } else {
            Some(parse_block(p))
        }
    } else {
        None
    };

    let end = else_block
        .as_ref()
        .and_then(|b| b.statements.last().map(|s| s.span))
        .or(elif_clauses
            .last()
            .and_then(|(_, b)| b.statements.last().map(|s| s.span)))
        .or(then_block.statements.last().map(|s| s.span))
        .unwrap_or(start);

    Spanned::new(
        Stmt::If(IfStmt {
            condition,
            then_block,
            elif_clauses,
            else_block,
        }),
        start.merge(end),
    )
}

// ── For Statement ────────────────────────────────────────

fn parse_for_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // for

    // Java-style: for (init; cond; update)
    if p.at(TokenKind::LParen) {
        p.advance(); // (

        // init
        let init = if p.at(TokenKind::Semicolon) {
            None
        } else {
            let stmt = parse_stmt(p);
            Some(Box::new(stmt))
        };
        p.expect(TokenKind::Semicolon);

        // condition
        let condition = if p.at(TokenKind::Semicolon) {
            None
        } else {
            Some(expr::parse_expression(p))
        };
        p.expect(TokenKind::Semicolon);

        // update
        let update = if p.at(TokenKind::RParen) {
            None
        } else {
            Some(expr::parse_expression(p))
        };
        p.expect(TokenKind::RParen);

        let body = parse_block(p);
        let end = body.statements.last().map(|s| s.span).unwrap_or(start);

        return Spanned::new(
            Stmt::For(ForStmt::ForClassic {
                init,
                condition,
                update,
                body,
            }),
            start.merge(end),
        );
    }

    // Python-style: for target in iter:
    // Parse target at high precedence to avoid consuming 'in' as binary op
    let target = expr::parse_expr(p, expr::Prec::Shift);
    p.expect(TokenKind::KwIn);
    let iter = expr::parse_expression(p);
    let body = parse_block(p);

    let end = body.statements.last().map(|s| s.span).unwrap_or(start);

    Spanned::new(
        Stmt::For(ForStmt::ForIn { target, iter, body }),
        start.merge(end),
    )
}

// ── While Statement ──────────────────────────────────────

fn parse_while_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // while

    let has_paren = p.eat(TokenKind::LParen);
    let condition = expr::parse_expression(p);
    if has_paren {
        p.expect(TokenKind::RParen);
    }

    let body = parse_block(p);
    let end = body.statements.last().map(|s| s.span).unwrap_or(start);

    Spanned::new(Stmt::While(WhileStmt { condition, body }), start.merge(end))
}

fn parse_do_while(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // do
    let body = parse_block(p);
    p.skip_terminators();
    p.expect(TokenKind::KwWhile);

    let has_paren = p.eat(TokenKind::LParen);
    let condition = expr::parse_expression(p);
    if has_paren {
        p.expect(TokenKind::RParen);
    }

    let end = condition.span;

    Spanned::new(
        Stmt::DoWhile(DoWhileStmt { body, condition }),
        start.merge(end),
    )
}

// ── Try Statement ────────────────────────────────────────

fn parse_try_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // try
    let body = parse_block(p);

    p.skip_terminators();

    let mut catch_clauses = Vec::new();

    // Python-style: except
    while p.at(TokenKind::KwExcept) {
        p.advance(); // except
        let exception_type =
            if !p.at(TokenKind::Colon) && !p.at(TokenKind::LBrace) && !p.at(TokenKind::KwAs) {
                Some(crate::types::parse_type_expr(p))
            } else {
                None
            };

        let name = if p.eat(TokenKind::KwAs) {
            let n_span = p.expect(TokenKind::Identifier);
            let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
            Some(Spanned::new(n, n_span))
        } else {
            None
        };

        let catch_body = parse_block(p);
        catch_clauses.push(CatchClause {
            exception_type,
            name,
            body: catch_body,
        });
        p.skip_terminators();
    }

    // Java-style: catch
    while p.at(TokenKind::KwCatch) {
        p.advance(); // catch
        p.expect(TokenKind::LParen);

        let exception_type = if !p.at(TokenKind::Identifier) || p.peek_nth(1) == TokenKind::RParen {
            None
        } else {
            Some(crate::types::parse_type_expr(p))
        };

        let name = if p.at(TokenKind::Identifier) {
            let n_span = p.expect(TokenKind::Identifier);
            let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
            Some(Spanned::new(n, n_span))
        } else {
            None
        };

        p.expect(TokenKind::RParen);
        let catch_body = parse_block(p);
        catch_clauses.push(CatchClause {
            exception_type,
            name,
            body: catch_body,
        });
        p.skip_terminators();
    }

    let finally_block = if p.at(TokenKind::KwFinally) {
        p.advance(); // finally
        Some(parse_block(p))
    } else {
        None
    };

    let end = finally_block
        .as_ref()
        .and_then(|b| b.statements.last().map(|s| s.span))
        .or(catch_clauses
            .last()
            .and_then(|c| c.body.statements.last().map(|s| s.span)))
        .or(body.statements.last().map(|s| s.span))
        .unwrap_or(start);

    Spanned::new(
        Stmt::Try(TryStmt {
            body,
            catch_clauses,
            finally_block,
        }),
        start.merge(end),
    )
}

// ── With Statement ───────────────────────────────────────

fn parse_with_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // with
    let mut items = Vec::new();

    loop {
        let context = expr::parse_expression(p);
        let alias = if p.eat(TokenKind::KwAs) {
            let n_span = p.expect(TokenKind::Identifier);
            let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
            Some(Spanned::new(n, n_span))
        } else {
            None
        };
        items.push(WithItem { context, alias });
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }

    let body = parse_block(p);
    let end = body.statements.last().map(|s| s.span).unwrap_or(start);

    Spanned::new(Stmt::With(WithStmt { items, body }), start.merge(end))
}

// ── Import Statement ─────────────────────────────────────

fn parse_import_stmt(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // import

    // `import static ...`
    if p.at(TokenKind::KwStatic) {
        p.advance();
        let path = parse_dotted_path(p);
        let end = path.last().map(|s| s.span).unwrap_or(start);
        return Spanned::new(Stmt::Import(ImportStmt::Static { path }), start.merge(end));
    }

    // `import foo.bar [as baz]`
    let path = parse_dotted_path(p);
    let alias = if p.eat(TokenKind::KwAs) {
        let n_span = p.expect(TokenKind::Identifier);
        let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
        Some(Spanned::new(n, n_span))
    } else {
        None
    };

    let end = alias
        .as_ref()
        .map(|a| a.span)
        .or(path.last().map(|s| s.span))
        .unwrap_or(start);

    Spanned::new(
        Stmt::Import(ImportStmt::Simple { path, alias }),
        start.merge(end),
    )
}

fn parse_from_import(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // from
    let path = parse_dotted_path(p);
    p.expect(TokenKind::KwImport);

    let names = if p.eat(TokenKind::Star) {
        ImportNames::Wildcard
    } else {
        let mut aliases = Vec::new();
        loop {
            let n_span = p.expect(TokenKind::Identifier);
            let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
            let alias = if p.eat(TokenKind::KwAs) {
                let a_span = p.expect(TokenKind::Identifier);
                let a = p.source[a_span.start as usize..a_span.end as usize].to_string();
                Some(Spanned::new(a, a_span))
            } else {
                None
            };
            aliases.push(ImportAlias {
                name: Spanned::new(n, n_span),
                alias,
            });
            if !p.eat(TokenKind::Comma) {
                break;
            }
        }
        ImportNames::Named(aliases)
    };

    let end = match &names {
        ImportNames::Named(aliases) => aliases
            .last()
            .map(|a| a.alias.as_ref().map(|al| al.span).unwrap_or(a.name.span))
            .unwrap_or(start),
        ImportNames::Wildcard => p.current_span(),
    };

    Spanned::new(
        Stmt::Import(ImportStmt::From { path, names }),
        start.merge(end),
    )
}

fn parse_dotted_path(p: &mut Parser<'_>) -> Vec<Spanned<String>> {
    let mut path = Vec::new();
    let n_span = p.expect(TokenKind::Identifier);
    let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
    path.push(Spanned::new(n, n_span));

    while p.at(TokenKind::Dot) && p.peek_nth(1) == TokenKind::Identifier {
        p.advance(); // .
        let n_span = p.expect(TokenKind::Identifier);
        let n = p.source[n_span.start as usize..n_span.end as usize].to_string();
        path.push(Spanned::new(n, n_span));
    }

    path
}

// ── Simple Statements ────────────────────────────────────

fn parse_return(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // return

    // Check if there's an expression following
    if matches!(
        p.peek_kind(),
        TokenKind::Newline
            | TokenKind::Semicolon
            | TokenKind::Dedent
            | TokenKind::RBrace
            | TokenKind::Eof
    ) {
        return Spanned::new(Stmt::Return(None), start);
    }

    let expr = expr::parse_expression(p);
    let span = start.merge(expr.span);
    Spanned::new(Stmt::Return(Some(expr)), span)
}

fn parse_throw(p: &mut Parser<'_>, _syntax: SyntaxOrigin) -> Spanned<Stmt> {
    let start = p.advance().span; // raise/throw
    let expr = expr::parse_expression(p);
    let span = start.merge(expr.span);
    Spanned::new(Stmt::Throw(expr), span)
}

fn parse_assert(p: &mut Parser<'_>) -> Spanned<Stmt> {
    let start = p.advance().span; // assert
    let condition = expr::parse_expression(p);
    let msg = if p.eat(TokenKind::Comma) {
        Some(expr::parse_expression(p))
    } else {
        None
    };
    let end = msg.as_ref().map(|m| m.span).unwrap_or(condition.span);
    Spanned::new(Stmt::Assert(condition, msg), start.merge(end))
}
