// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # UniLang Lexer
//!
//! Tokenizes `.uniL` source files into a stream of tokens. Handles:
//! - The union of Python and Java token sets
//! - Indentation tracking (INDENT/DEDENT tokens)
//! - Automatic semicolon insertion (Newline tokens)
//! - All string literal types (single, double, triple, f-strings, raw)
//! - All numeric literal formats (decimal, hex, octal, binary, float)

pub mod indent;
pub mod keywords;
pub mod numbers;
pub mod scanner;
pub mod strings;
pub mod token;

use indent::{IndentChange, IndentTracker};
use scanner::Scanner;
use strings::StringScanResult;
use token::{Token, TokenKind};
use unilang_common::error::{codes, Diagnostic, DiagnosticBag};
use unilang_common::span::{SourceId, Span};

/// The UniLang lexer. Produces a stream of tokens from source text.
pub struct Lexer<'src> {
    scanner: Scanner<'src>,
    source_id: SourceId,
    indent: IndentTracker,
    diagnostics: DiagnosticBag,
    /// The kind of the last non-trivia token emitted (for ASI).
    prev_token: Option<TokenKind>,
    /// Byte offset where the last emitted token ended (for gap measurement).
    prev_token_end: u32,
    /// Whether we are at the very beginning of a line (for indentation).
    at_line_start: bool,
    /// Nesting depth of `( )` and `[ ]` — suppresses newline emission.
    paren_depth: u32,
    /// Whether we have emitted EOF.
    done: bool,
    /// Queue of tokens to emit before the next scanned token.
    queue: Vec<Token>,
}

impl<'src> Lexer<'src> {
    pub fn new(source_id: SourceId, source: &'src str) -> Self {
        Self {
            scanner: Scanner::new(source),
            source_id,
            indent: IndentTracker::new(),
            diagnostics: DiagnosticBag::new(),
            prev_token: None,
            prev_token_end: 0,
            at_line_start: true,
            paren_depth: 0,
            done: false,
            queue: Vec::new(),
        }
    }

    /// Consume the lexer, returning all tokens at once.
    pub fn tokenize(mut self) -> (Vec<Token>, DiagnosticBag) {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        (tokens, self.diagnostics)
    }

    /// Get the accumulated diagnostics.
    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Token {
        // Drain queued tokens first (INDENT, DEDENT, NEWLINE).
        if let Some(tok) = self.queue.pop() {
            self.record_prev(&tok);
            return tok;
        }

        if self.done {
            return Token::new(TokenKind::Eof, Span::empty(self.scanner.pos() as u32));
        }

        loop {
            let pos = self.scanner.pos() as u32;

            // Handle start-of-line: measure indentation.
            if self.at_line_start && !self.scanner.is_at_end() {
                self.at_line_start = false;
                let indent_level = self.measure_indent();

                // Only emit INDENT/DEDENT when outside ( ) [ ] — inside brackets,
                // indentation is meaningless (multi-line expressions are allowed).
                let change = if self.paren_depth == 0 {
                    self.indent.process_line_indent(indent_level)
                } else {
                    // Still advance the indent tracker's state so it doesn't get confused
                    // when we exit the bracket, but don't emit any tokens.
                    let _ = self.indent.process_line_indent(indent_level);
                    IndentChange::None
                };
                let span = Span::new(pos, self.scanner.pos() as u32);

                match change {
                    IndentChange::Indent => {
                        self.queue.push(Token::new(TokenKind::Indent, span));
                    }
                    IndentChange::Dedent(n) => {
                        for _ in 0..n {
                            self.queue.push(Token::new(TokenKind::Dedent, span));
                        }
                    }
                    IndentChange::None => {}
                }

                // If we queued indent/dedent tokens, return the first one.
                // The queue is used as a stack, so reverse the order.
                if !self.queue.is_empty() {
                    self.queue.reverse();
                    if let Some(tok) = self.queue.pop() {
                        self.record_prev(&tok);
                        return tok;
                    }
                }
            }

            // Skip whitespace (not newlines).
            self.skip_horizontal_whitespace();

            if self.scanner.is_at_end() {
                return self.emit_eof();
            }

            let start = self.scanner.pos() as u32;
            let ch = self.scanner.peek().unwrap();

            // ── Newlines ─────────────────────────────────────
            if ch == '\n' || ch == '\r' {
                self.scanner.advance();
                if ch == '\r' {
                    self.scanner.eat_if(|c| c == '\n');
                }
                self.at_line_start = true;

                // ASI: emit Newline if previous token can end a statement
                // and we're not inside parens/brackets.
                if self.paren_depth == 0 {
                    if let Some(prev) = self.prev_token {
                        if prev.can_end_statement() {
                            let span = Span::new(start, self.scanner.pos() as u32);
                            let tok = Token::new(TokenKind::Newline, span);
                            self.record_prev(&tok);
                            return tok;
                        }
                    }
                }
                // Skip non-significant newlines and continue.
                continue;
            }

            // ── Comments ─────────────────────────────────────
            // # Python-style line comment
            if ch == '#' {
                self.skip_line_comment();
                continue;
            }

            // // Java-style line comment or //= or // (floor div)
            if ch == '/' {
                if let Some(next) = self.scanner.peek_nth(1) {
                    if next == '/' {
                        // Could be floor division (//) or line comment
                        // Check for //= (floor div assign)
                        if self.scanner.peek_nth(2) == Some('=') {
                            self.scanner.advance_by(3);
                            let tok = Token::new(
                                TokenKind::DoubleSlashEq,
                                Span::new(start, self.scanner.pos() as u32),
                            );
                            self.record_prev(&tok);
                            return tok;
                        }
                        // Disambiguation: if prev token can appear before //,
                        // treat as floor division. Otherwise, treat as comment.
                        if self.is_floor_div_context(start) {
                            self.scanner.advance_by(2);
                            let tok = Token::new(
                                TokenKind::DoubleSlash,
                                Span::new(start, self.scanner.pos() as u32),
                            );
                            self.record_prev(&tok);
                            return tok;
                        }
                        // Line comment
                        self.skip_line_comment();
                        continue;
                    }
                    if next == '*' {
                        // Block comment /* ... */
                        self.skip_block_comment(start);
                        continue;
                    }
                }
            }

            // ── String literals ──────────────────────────────
            if ch == '"' || ch == '\'' {
                return self.scan_string_token(start, ch);
            }

            // f-string, r-string, b-string prefixes
            if (ch == 'f' || ch == 'r' || ch == 'b' || ch == 'F' || ch == 'R' || ch == 'B')
                && matches!(self.scanner.peek_nth(1), Some('"' | '\''))
            {
                return self.scan_prefixed_string(start, ch);
            }

            // ── Numeric literals ─────────────────────────────
            if ch.is_ascii_digit() {
                let kind = numbers::scan_number(&mut self.scanner);
                let tok = Token::new(kind, Span::new(start, self.scanner.pos() as u32));
                self.record_prev(&tok);
                return tok;
            }

            // ── Identifiers and keywords ─────────────────────
            if ch.is_alphabetic() || ch == '_' {
                return self.scan_identifier(start);
            }

            // ── Operators and punctuation ────────────────────
            return self.scan_operator(start, ch);
        }
    }

    // ── Helpers ──────────────────────────────────────────────

    fn record_prev(&mut self, tok: &Token) {
        self.prev_token = Some(tok.kind);
        self.prev_token_end = tok.span.end;
    }

    fn emit_eof(&mut self) -> Token {
        let pos = self.scanner.pos() as u32;

        // Emit final NEWLINE if needed.
        if self.paren_depth == 0 {
            if let Some(prev) = self.prev_token {
                if prev.can_end_statement() && prev != TokenKind::Newline {
                    let nl = Token::new(TokenKind::Newline, Span::empty(pos));
                    self.record_prev(&nl);
                    let eof_dedents = self.indent.flush_eof();
                    for _ in 0..eof_dedents {
                        self.queue
                            .push(Token::new(TokenKind::Dedent, Span::empty(pos)));
                    }
                    self.queue
                        .push(Token::new(TokenKind::Eof, Span::empty(pos)));
                    self.queue.reverse();
                    return nl;
                }
            }
        }

        // Emit remaining DEDENTs.
        let eof_dedents = self.indent.flush_eof();
        if eof_dedents > 0 {
            for _ in 0..eof_dedents {
                self.queue
                    .push(Token::new(TokenKind::Dedent, Span::empty(pos)));
            }
            self.queue
                .push(Token::new(TokenKind::Eof, Span::empty(pos)));
            self.queue.reverse();
            let tok = self.queue.pop().unwrap();
            self.record_prev(&tok);
            self.done = true;
            return tok;
        }

        self.done = true;
        Token::new(TokenKind::Eof, Span::empty(pos))
    }

    fn measure_indent(&mut self) -> u32 {
        let mut spaces = 0u32;
        while let Some(ch) = self.scanner.peek() {
            match ch {
                ' ' => {
                    spaces += 1;
                    self.scanner.advance();
                }
                '\t' => {
                    spaces += 4; // Tabs count as 4 spaces.
                    self.scanner.advance();
                }
                _ => break,
            }
        }
        spaces
    }

    fn skip_horizontal_whitespace(&mut self) {
        self.scanner.eat_while(|c| c == ' ' || c == '\t');
    }

    fn skip_line_comment(&mut self) {
        self.scanner.eat_while(|c| c != '\n' && c != '\r');
    }

    fn skip_block_comment(&mut self, start: u32) {
        self.scanner.advance_by(2); // skip /*
        let mut depth = 1u32;
        while !self.scanner.is_at_end() && depth > 0 {
            if self.scanner.check_str("/*") {
                self.scanner.advance_by(2);
                depth += 1;
            } else if self.scanner.check_str("*/") {
                self.scanner.advance_by(2);
                depth -= 1;
            } else {
                self.scanner.advance();
            }
        }
        if depth > 0 {
            self.diagnostics.report(
                Diagnostic::error("unterminated block comment")
                    .with_code(codes::UNTERMINATED_COMMENT)
                    .with_label(
                        Span::new(start, self.scanner.pos() as u32),
                        self.source_id,
                        "comment starts here",
                    ),
            );
        }
    }

    /// Determine if `//` should be floor division rather than a comment.
    ///
    /// Floor division requires:
    ///   1. preceding token is expression-ending, AND
    ///   2. at most 1 character of whitespace between that token and `//`
    ///      (covers `a//b` and `a // b` but NOT `x  // trailing comment`).
    fn is_floor_div_context(&self, slash_start: u32) -> bool {
        let is_expr_ending = matches!(
            self.prev_token,
            Some(
                TokenKind::Identifier
                    | TokenKind::IntLiteral
                    | TokenKind::FloatLiteral
                    | TokenKind::RParen
                    | TokenKind::RBracket
                    | TokenKind::RBrace
            )
        );
        if !is_expr_ending {
            return false;
        }
        let gap = slash_start.saturating_sub(self.prev_token_end);
        gap <= 1
    }

    fn scan_string_token(&mut self, start: u32, quote: char) -> Token {
        self.scanner.advance(); // consume opening quote
        let result = strings::scan_string(&mut self.scanner, quote);
        match result {
            StringScanResult::Complete(kind) => {
                let tok = Token::new(kind, Span::new(start, self.scanner.pos() as u32));
                self.record_prev(&tok);
                tok
            }
            StringScanResult::Unterminated => {
                self.diagnostics.report(
                    Diagnostic::error("unterminated string literal")
                        .with_code(codes::UNTERMINATED_STRING)
                        .with_label(
                            Span::new(start, self.scanner.pos() as u32),
                            self.source_id,
                            "string starts here",
                        ),
                );
                let tok = Token::new(
                    TokenKind::Error,
                    Span::new(start, self.scanner.pos() as u32),
                );
                self.record_prev(&tok);
                tok
            }
        }
    }

    fn scan_prefixed_string(&mut self, start: u32, prefix: char) -> Token {
        self.scanner.advance(); // consume prefix
        let quote = self.scanner.peek().unwrap();
        self.scanner.advance(); // consume opening quote

        let result = match prefix.to_ascii_lowercase() {
            'f' => strings::scan_fstring(&mut self.scanner, quote),
            'r' => strings::scan_raw_string(&mut self.scanner, quote),
            'b' => strings::scan_string(&mut self.scanner, quote),
            _ => unreachable!(),
        };

        match result {
            StringScanResult::Complete(kind) => {
                let tok = Token::new(kind, Span::new(start, self.scanner.pos() as u32));
                self.record_prev(&tok);
                tok
            }
            StringScanResult::Unterminated => {
                self.diagnostics.report(
                    Diagnostic::error("unterminated string literal")
                        .with_code(codes::UNTERMINATED_STRING)
                        .with_label(
                            Span::new(start, self.scanner.pos() as u32),
                            self.source_id,
                            "string starts here",
                        ),
                );
                let tok = Token::new(
                    TokenKind::Error,
                    Span::new(start, self.scanner.pos() as u32),
                );
                self.record_prev(&tok);
                tok
            }
        }
    }

    fn scan_identifier(&mut self, start: u32) -> Token {
        self.scanner.eat_while(|c| c.is_alphanumeric() || c == '_');
        let text = self.scanner.slice_from(start as usize);
        let kind = keywords::lookup_keyword(text).unwrap_or(TokenKind::Identifier);
        let tok = Token::new(kind, Span::new(start, self.scanner.pos() as u32));
        self.record_prev(&tok);
        tok
    }

    fn scan_operator(&mut self, start: u32, ch: char) -> Token {
        let kind = match ch {
            '+' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '>') {
                    TokenKind::Arrow
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '*') {
                    if self.scanner.eat_if(|c| c == '=') {
                        TokenKind::DoubleStarEq
                    } else {
                        TokenKind::DoubleStar
                    }
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            '=' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::EqEq
                } else if self.scanner.eat_if(|c| c == '>') {
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::NotEq
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '<') {
                    if self.scanner.eat_if(|c| c == '=') {
                        TokenKind::LShiftEq
                    } else {
                        TokenKind::LShift
                    }
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '>') {
                    if self.scanner.eat_if(|c| c == '>') {
                        TokenKind::UnsignedRShift
                    } else if self.scanner.eat_if(|c| c == '=') {
                        TokenKind::RShiftEq
                    } else {
                        TokenKind::RShift
                    }
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '&') {
                    TokenKind::AmpAmp
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::AmpEq
                } else {
                    TokenKind::Amp
                }
            }
            '|' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '|') {
                    TokenKind::PipePipe
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::CaretEq
                } else {
                    TokenKind::Caret
                }
            }
            '~' => {
                self.scanner.advance();
                TokenKind::Tilde
            }
            '(' => {
                self.scanner.advance();
                self.paren_depth += 1;
                TokenKind::LParen
            }
            ')' => {
                self.scanner.advance();
                self.paren_depth = self.paren_depth.saturating_sub(1);
                TokenKind::RParen
            }
            '[' => {
                self.scanner.advance();
                self.paren_depth += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.scanner.advance();
                self.paren_depth = self.paren_depth.saturating_sub(1);
                TokenKind::RBracket
            }
            '{' => {
                self.scanner.advance();
                self.indent.open_brace();
                TokenKind::LBrace
            }
            '}' => {
                self.scanner.advance();
                self.indent.close_brace();
                TokenKind::RBrace
            }
            ',' => {
                self.scanner.advance();
                TokenKind::Comma
            }
            ';' => {
                self.scanner.advance();
                TokenKind::Semicolon
            }
            ':' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == ':') {
                    TokenKind::DoubleColon
                } else if self.scanner.eat_if(|c| c == '=') {
                    TokenKind::ColonEq
                } else {
                    TokenKind::Colon
                }
            }
            '.' => {
                self.scanner.advance();
                if self.scanner.check_str("..") {
                    self.scanner.advance_by(2);
                    TokenKind::Ellipsis
                } else {
                    TokenKind::Dot
                }
            }
            '@' => {
                self.scanner.advance();
                TokenKind::At
            }
            '?' => {
                self.scanner.advance();
                if self.scanner.eat_if(|c| c == '.') {
                    TokenKind::QuestionDot
                } else if self.scanner.eat_if(|c| c == '?') {
                    TokenKind::DoubleQuestion
                } else {
                    TokenKind::Question
                }
            }
            _ => {
                self.scanner.advance();
                self.diagnostics.report(
                    Diagnostic::error(format!("unexpected character '{}'", ch))
                        .with_code(codes::UNEXPECTED_CHAR)
                        .with_label(
                            Span::new(start, self.scanner.pos() as u32),
                            self.source_id,
                            "here",
                        ),
                );
                TokenKind::Error
            }
        };

        let tok = Token::new(kind, Span::new(start, self.scanner.pos() as u32));
        self.record_prev(&tok);
        tok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<TokenKind> {
        let lexer = Lexer::new(SourceId(0), input);
        let (tokens, _) = lexer.tokenize();
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty() {
        assert_eq!(lex(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn test_simple_assignment() {
        let kinds = lex("x = 42");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::Eq,
                TokenKind::IntLiteral,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_python_function() {
        let kinds = lex("def foo(x):\n    return x");
        // After `:`, no Newline is emitted (colon continues the line).
        // INDENT appears at the start of the indented line.
        assert_eq!(
            kinds,
            vec![
                TokenKind::KwDef,
                TokenKind::Identifier, // foo
                TokenKind::LParen,
                TokenKind::Identifier, // x
                TokenKind::RParen,
                TokenKind::Colon,
                TokenKind::Indent,
                TokenKind::KwReturn,
                TokenKind::Identifier, // x
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_java_method() {
        let kinds = lex("public int add(int a, int b) { return a + b; }");
        assert_eq!(
            kinds,
            vec![
                TokenKind::KwPublic,
                TokenKind::Identifier, // int
                TokenKind::Identifier, // add
                TokenKind::LParen,
                TokenKind::Identifier, // int
                TokenKind::Identifier, // a
                TokenKind::Comma,
                TokenKind::Identifier, // int
                TokenKind::Identifier, // b
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::KwReturn,
                TokenKind::Identifier, // a
                TokenKind::Plus,
                TokenKind::Identifier, // b
                TokenKind::Semicolon,
                TokenKind::RBrace,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literals() {
        let kinds = lex(r#""hello""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral, TokenKind::Newline, TokenKind::Eof,]
        );
    }

    #[test]
    fn test_fstring() {
        let kinds = lex(r#"f"hello {name}""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral, TokenKind::Newline, TokenKind::Eof,]
        );
    }

    #[test]
    fn test_operators() {
        let kinds = lex("a ** b");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::DoubleStar,
                TokenKind::Identifier,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_walrus() {
        let kinds = lex("x := 5");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::ColonEq,
                TokenKind::IntLiteral,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_arrow_and_fat_arrow() {
        let kinds = lex("-> =>");
        assert_eq!(
            kinds,
            vec![TokenKind::Arrow, TokenKind::FatArrow, TokenKind::Eof,]
        );
    }

    #[test]
    fn test_comments_skipped() {
        // # comment
        let kinds = lex("x = 1 # comment\ny = 2");
        assert!(kinds.contains(&TokenKind::Identifier));
        assert!(!kinds.contains(&TokenKind::Hash));
    }

    #[test]
    fn test_indentation_in_braces_suppressed() {
        let kinds = lex("if (x) {\n    foo\n}");
        // Should NOT contain Indent/Dedent inside braces
        assert!(!kinds.contains(&TokenKind::Indent));
        assert!(!kinds.contains(&TokenKind::Dedent));
    }

    #[test]
    fn test_paren_suppresses_newline() {
        let kinds = lex("foo(\n  a,\n  b\n)");
        // Inside parens, newlines should not produce Newline tokens
        let newline_count = kinds.iter().filter(|k| **k == TokenKind::Newline).count();
        // Only the final Newline after ) should be emitted
        assert_eq!(newline_count, 1);
    }

    #[test]
    fn test_ellipsis() {
        let kinds = lex("...");
        assert_eq!(kinds, vec![TokenKind::Ellipsis, TokenKind::Eof]);
    }

    #[test]
    fn test_optional_chaining() {
        let kinds = lex("x?.y");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::QuestionDot,
                TokenKind::Identifier,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_unsigned_right_shift() {
        let kinds = lex("a >>> b");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::UnsignedRShift,
                TokenKind::Identifier,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }
}
