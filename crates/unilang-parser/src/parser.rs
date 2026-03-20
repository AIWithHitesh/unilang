// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Core parser struct and infrastructure.

use unilang_common::error::{codes, Diagnostic, DiagnosticBag};
use unilang_common::span::{SourceId, Span, Spanned};
use unilang_lexer::token::{Token, TokenKind};

use crate::ast::*;

/// The UniLang parser — transforms a token stream into an AST.
pub struct Parser<'src> {
    pub(crate) tokens: Vec<Token>,
    pub(crate) source: &'src str,
    pub(crate) source_id: SourceId,
    pub(crate) pos: usize,
    diagnostics: DiagnosticBag,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: Vec<Token>, source: &'src str, source_id: SourceId) -> Self {
        Self {
            tokens,
            source,
            source_id,
            pos: 0,
            diagnostics: DiagnosticBag::new(),
        }
    }

    // ── Accessors ────────────────────────────────────────────

    pub fn diagnostics(self) -> DiagnosticBag {
        self.diagnostics
    }

    // ── Token helpers ────────────────────────────────────────

    pub(crate) fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    pub(crate) fn peek_kind(&self) -> TokenKind {
        self.peek().kind
    }

    /// Look ahead `n` tokens (0 = current).
    pub(crate) fn peek_nth(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    pub(crate) fn at(&self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    #[allow(dead_code)]
    pub(crate) fn check(&self, kind: TokenKind) -> bool {
        self.at(kind)
    }

    pub(crate) fn at_eof(&self) -> bool {
        self.at(TokenKind::Eof)
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    pub(crate) fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, kind: TokenKind) -> Span {
        if self.at(kind) {
            let tok = self.advance();
            tok.span
        } else {
            let tok = self.peek();
            let span = tok.span;
            self.diagnostics.report(
                Diagnostic::error(format!("expected {}, found {}", kind, tok.kind))
                    .with_code(codes::UNEXPECTED_TOKEN)
                    .with_label(span, self.source_id, "here"),
            );
            span
        }
    }

    /// Get the text of a token from the source.
    #[allow(dead_code)]
    pub(crate) fn token_text(&self, tok: &Token) -> &'src str {
        let start = tok.span.start as usize;
        let end = tok.span.end as usize;
        &self.source[start..end.min(self.source.len())]
    }

    /// Get text for the current token without advancing.
    #[allow(dead_code)]
    pub(crate) fn current_text(&self) -> &'src str {
        self.token_text(self.peek())
    }

    /// Get the text of a span from the source.
    #[allow(dead_code)]
    pub(crate) fn span_text(&self, span: Span) -> &'src str {
        let start = span.start as usize;
        let end = span.end as usize;
        &self.source[start..end.min(self.source.len())]
    }

    pub(crate) fn current_span(&self) -> Span {
        self.peek().span
    }

    /// Skip newlines and semicolons (statement terminators).
    pub(crate) fn skip_terminators(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Semicolon) {
            self.advance();
        }
    }

    /// Skip a single optional statement terminator.
    #[allow(dead_code)]
    pub(crate) fn skip_terminator(&mut self) {
        if matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Semicolon) {
            self.advance();
        }
    }

    // ── Error recovery ───────────────────────────────────────

    /// Skip tokens until we reach a statement boundary.
    pub(crate) fn synchronize(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::Newline | TokenKind::Semicolon | TokenKind::Dedent | TokenKind::Eof => {
                    return;
                }
                // Statement-starting keywords
                TokenKind::KwDef
                | TokenKind::KwClass
                | TokenKind::KwIf
                | TokenKind::KwFor
                | TokenKind::KwWhile
                | TokenKind::KwReturn
                | TokenKind::KwImport
                | TokenKind::KwFrom
                | TokenKind::KwTry
                | TokenKind::KwWith
                | TokenKind::KwRaise
                | TokenKind::KwThrow
                | TokenKind::KwBreak
                | TokenKind::KwContinue
                | TokenKind::KwPass
                | TokenKind::KwPublic
                | TokenKind::KwPrivate
                | TokenKind::KwProtected
                | TokenKind::KwVar
                | TokenKind::KwVal
                | TokenKind::KwConst
                | TokenKind::KwDo
                | TokenKind::At => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn error_expr(&mut self, msg: &str) -> Spanned<Expr> {
        let span = self.current_span();
        self.diagnostics.report(
            Diagnostic::error(msg)
                .with_code(codes::EXPECTED_EXPRESSION)
                .with_label(span, self.source_id, "here"),
        );
        Spanned::new(Expr::Error, span)
    }

    pub(crate) fn error_stmt(&mut self, msg: &str) -> Spanned<Stmt> {
        let span = self.current_span();
        self.diagnostics.report(
            Diagnostic::error(msg)
                .with_code(codes::UNEXPECTED_TOKEN)
                .with_label(span, self.source_id, "here"),
        );
        self.synchronize();
        Spanned::new(Stmt::Error, span)
    }

    // ── Main entry ───────────────────────────────────────────

    pub fn parse(&mut self) -> Module {
        let mut statements = Vec::new();
        self.skip_terminators();
        while !self.at_eof() {
            let stmt = crate::stmt::parse_stmt(self);
            statements.push(stmt);
            self.skip_terminators();
        }
        Module {
            source_id: self.source_id,
            statements,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(source: &str) -> (Module, DiagnosticBag) {
        crate::parse(SourceId(0), source)
    }

    fn parse_ok(source: &str) -> Module {
        let (module, diag) = parse_source(source);
        if diag.has_errors() {
            for d in diag.diagnostics() {
                eprintln!("ERROR: {}", d.message);
            }
            panic!("unexpected parse errors");
        }
        module
    }

    #[test]
    fn test_parse_int_literal() {
        let m = parse_ok("42");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::Expr(Expr::IntLit(v)) => assert_eq!(*v, 42),
            other => panic!("expected IntLit, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let m = parse_ok(r#""hello""#);
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::Expr(Expr::StringLit(s)) => assert_eq!(s, "hello"),
            other => panic!("expected StringLit, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_binary_op() {
        let m = parse_ok("a + b");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::Expr(Expr::BinaryOp(_, op, _)) => assert_eq!(*op, BinOp::Add),
            other => panic!("expected BinaryOp, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let m = parse_ok("foo(1, 2)");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::Expr(Expr::Call(callee, args)) => {
                match &callee.node {
                    Expr::Ident(name) => assert_eq!(name, "foo"),
                    other => panic!("expected Ident, got {:?}", other),
                }
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected Call, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_python_function() {
        let m = parse_ok("def foo(x):\n    return x");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::FunctionDecl(f) => {
                assert_eq!(f.name.node, "foo");
                assert_eq!(f.params.len(), 1);
                assert_eq!(f.body.style, BlockStyle::Indentation);
            }
            other => panic!("expected FunctionDecl, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_java_method() {
        let m = parse_ok("public int add(int a, int b) { return a + b; }");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::FunctionDecl(f) => {
                assert_eq!(f.name.node, "add");
                assert_eq!(f.visibility, Visibility::Public);
                assert_eq!(f.params.len(), 2);
                assert_eq!(f.body.style, BlockStyle::Braces);
            }
            other => panic!("expected FunctionDecl, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_python_class() {
        let m = parse_ok("class Foo:\n    pass");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::ClassDecl(c) => {
                assert_eq!(c.name.node, "Foo");
            }
            other => panic!("expected ClassDecl, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_java_class() {
        let m = parse_ok("public class Bar { }");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::ClassDecl(c) => {
                assert_eq!(c.name.node, "Bar");
                assert_eq!(c.visibility, Visibility::Public);
            }
            other => panic!("expected ClassDecl, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_if_elif_else_python() {
        let m = parse_ok("if x:\n    a\nelif y:\n    b\nelse:\n    c");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::If(if_stmt) => {
                assert_eq!(if_stmt.elif_clauses.len(), 1);
                assert!(if_stmt.else_block.is_some());
            }
            other => panic!("expected If, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_if_else_java() {
        let m = parse_ok("if (x) { a; } else { b; }");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::If(if_stmt) => {
                assert_eq!(if_stmt.then_block.style, BlockStyle::Braces);
                assert!(if_stmt.else_block.is_some());
            }
            other => panic!("expected If, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_for_in_loop() {
        let m = parse_ok("for x in items:\n    pass");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::For(ForStmt::ForIn { .. }) => {}
            other => panic!("expected ForIn, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_import_simple() {
        let m = parse_ok("import foo.bar");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::Import(ImportStmt::Simple { path, .. }) => {
                assert_eq!(path.len(), 2);
            }
            other => panic!("expected Import Simple, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_from_import() {
        let m = parse_ok("from foo.bar import baz");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::Import(ImportStmt::From { path, names }) => {
                assert_eq!(path.len(), 2);
                match names {
                    ImportNames::Named(aliases) => assert_eq!(aliases.len(), 1),
                    _ => panic!("expected Named"),
                }
            }
            other => panic!("expected Import From, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_var_decl() {
        let m = parse_ok("var x = 42");
        assert_eq!(m.statements.len(), 1);
        match &m.statements[0].node {
            Stmt::VarDecl(v) => {
                assert_eq!(v.name.node, "x");
                assert!(v.initializer.is_some());
            }
            other => panic!("expected VarDecl, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_nested_expr() {
        let m = parse_ok("a + b * c");
        assert_eq!(m.statements.len(), 1);
        // a + (b * c) due to precedence
        match &m.statements[0].node {
            Stmt::Expr(Expr::BinaryOp(left, BinOp::Add, right)) => {
                match &left.node {
                    Expr::Ident(name) => assert_eq!(name, "a"),
                    other => panic!("expected Ident, got {:?}", other),
                }
                match &right.node {
                    Expr::BinaryOp(_, BinOp::Mul, _) => {}
                    other => panic!("expected Mul, got {:?}", other),
                }
            }
            other => panic!("expected BinaryOp Add, got {:?}", other),
        }
    }
}
