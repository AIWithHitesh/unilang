// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

use std::fmt;
use unilang_common::span::Span;

/// A single token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// All token kinds in UniLang — the union of Python and Java token sets,
/// plus synthetic tokens for indentation and ASI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ── Literals ──────────────────────────────────────────
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    FStringStart,
    FStringMiddle,
    FStringEnd,
    TripleStringLiteral,
    BoolTrue,
    BoolFalse,
    NullNone, // None
    NullNull, // null

    // ── Identifier ────────────────────────────────────────
    Identifier,

    // ── Shared Keywords ──────────────────────────────────
    KwClass,
    KwIf,
    KwElse,
    KwFor,
    KwWhile,
    KwReturn,
    KwImport,
    KwTry,
    KwFinally,
    KwContinue,
    KwBreak,
    KwAssert,

    // ── Python-origin Keywords ───────────────────────────
    KwDef,
    KwElif,
    KwExcept,
    KwRaise,
    KwPass,
    KwYield,
    KwAsync,
    KwAwait,
    KwWith,
    KwAs,
    KwFrom,
    KwIn,
    KwIs,
    KwNot,
    KwAnd,
    KwOr,
    KwLambda,
    KwNonlocal,
    KwGlobal,
    KwDel,
    KwMatch,
    KwCase,

    // ── Java-origin Keywords ─────────────────────────────
    KwPublic,
    KwPrivate,
    KwProtected,
    KwStatic,
    KwFinal,
    KwAbstract,
    KwInterface,
    KwEnum,
    KwExtends,
    KwImplements,
    KwNew,
    KwThis,
    KwSuper,
    KwVoid,
    KwThrows,
    KwThrow,
    KwCatch,
    KwSynchronized,
    KwVolatile,
    KwTransient,
    KwNative,
    KwInstanceof,
    KwSwitch,
    KwDefault,
    KwDo,

    // ── UniLang-specific Keywords ────────────────────────
    KwBridge,
    KwVm,
    KwInterop,
    KwVal,
    KwVar,
    KwConst,

    // ── Arithmetic Operators ─────────────────────────────
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    DoubleSlash, // //
    Percent,     // %
    DoubleStar,  // **

    // ── Comparison Operators ─────────────────────────────
    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    Gt,    // >
    LtEq,  // <=
    GtEq,  // >=

    // ── Logical Operators ────────────────────────────────
    AmpAmp,   // &&
    PipePipe, // ||
    Bang,     // !

    // ── Bitwise Operators ────────────────────────────────
    Amp,            // &
    Pipe,           // |
    Caret,          // ^
    Tilde,          // ~
    LShift,         // <<
    RShift,         // >>
    UnsignedRShift, // >>>

    // ── Assignment Operators ─────────────────────────────
    Eq,            // =
    PlusEq,        // +=
    MinusEq,       // -=
    StarEq,        // *=
    SlashEq,       // /=
    DoubleSlashEq, // //=
    PercentEq,     // %=
    DoubleStarEq,  // **=
    AmpEq,         // &=
    PipeEq,        // |=
    CaretEq,       // ^=
    LShiftEq,      // <<=
    RShiftEq,      // >>=
    ColonEq,       // :=  (walrus operator)

    // ── Punctuation / Delimiters ─────────────────────────
    LParen,         // (
    RParen,         // )
    LBracket,       // [
    RBracket,       // ]
    LBrace,         // {
    RBrace,         // }
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    DoubleColon,    // ::
    Dot,            // .
    Ellipsis,       // ...
    At,             // @
    Arrow,          // ->
    FatArrow,       // =>
    Question,       // ?
    QuestionDot,    // ?.
    DoubleQuestion, // ??
    Hash,           // # (line comment starter in Python — handled as trivia,
    //    but token is available for directives)

    // ── Synthetic Tokens ─────────────────────────────────
    Indent,  // indentation level increased
    Dedent,  // indentation level decreased
    Newline, // significant newline (statement terminator from ASI)

    // ── Special ──────────────────────────────────────────
    Eof,
    Error,
}

impl TokenKind {
    /// Returns `true` if this token can end a statement (for ASI purposes).
    pub fn can_end_statement(self) -> bool {
        matches!(
            self,
            TokenKind::Identifier
                | TokenKind::IntLiteral
                | TokenKind::FloatLiteral
                | TokenKind::StringLiteral
                | TokenKind::TripleStringLiteral
                | TokenKind::FStringEnd
                | TokenKind::BoolTrue
                | TokenKind::BoolFalse
                | TokenKind::NullNone
                | TokenKind::NullNull
                | TokenKind::KwReturn
                | TokenKind::KwBreak
                | TokenKind::KwContinue
                | TokenKind::KwPass
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
        )
    }

    /// Returns `true` if this token prevents a newline from being
    /// a statement terminator (e.g., binary operators, open parens).
    pub fn continues_line(self) -> bool {
        matches!(
            self,
            TokenKind::Comma
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::DoubleSlash
                | TokenKind::Percent
                | TokenKind::DoubleStar
                | TokenKind::EqEq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::LtEq
                | TokenKind::GtEq
                | TokenKind::AmpAmp
                | TokenKind::PipePipe
                | TokenKind::Amp
                | TokenKind::Pipe
                | TokenKind::Caret
                | TokenKind::LShift
                | TokenKind::RShift
                | TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::StarEq
                | TokenKind::SlashEq
                | TokenKind::Dot
                | TokenKind::Arrow
                | TokenKind::FatArrow
                | TokenKind::Colon
                | TokenKind::QuestionDot
                | TokenKind::KwAnd
                | TokenKind::KwOr
                | TokenKind::KwNot
                | TokenKind::KwIn
                | TokenKind::KwIs
        )
    }

    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            TokenKind::KwClass
                | TokenKind::KwIf
                | TokenKind::KwElse
                | TokenKind::KwFor
                | TokenKind::KwWhile
                | TokenKind::KwReturn
                | TokenKind::KwImport
                | TokenKind::KwTry
                | TokenKind::KwFinally
                | TokenKind::KwContinue
                | TokenKind::KwBreak
                | TokenKind::KwAssert
                | TokenKind::KwDef
                | TokenKind::KwElif
                | TokenKind::KwExcept
                | TokenKind::KwRaise
                | TokenKind::KwPass
                | TokenKind::KwYield
                | TokenKind::KwAsync
                | TokenKind::KwAwait
                | TokenKind::KwWith
                | TokenKind::KwAs
                | TokenKind::KwFrom
                | TokenKind::KwIn
                | TokenKind::KwIs
                | TokenKind::KwNot
                | TokenKind::KwAnd
                | TokenKind::KwOr
                | TokenKind::KwLambda
                | TokenKind::KwNonlocal
                | TokenKind::KwGlobal
                | TokenKind::KwDel
                | TokenKind::KwMatch
                | TokenKind::KwCase
                | TokenKind::KwPublic
                | TokenKind::KwPrivate
                | TokenKind::KwProtected
                | TokenKind::KwStatic
                | TokenKind::KwFinal
                | TokenKind::KwAbstract
                | TokenKind::KwInterface
                | TokenKind::KwEnum
                | TokenKind::KwExtends
                | TokenKind::KwImplements
                | TokenKind::KwNew
                | TokenKind::KwThis
                | TokenKind::KwSuper
                | TokenKind::KwVoid
                | TokenKind::KwThrows
                | TokenKind::KwThrow
                | TokenKind::KwCatch
                | TokenKind::KwSynchronized
                | TokenKind::KwVolatile
                | TokenKind::KwTransient
                | TokenKind::KwNative
                | TokenKind::KwInstanceof
                | TokenKind::KwSwitch
                | TokenKind::KwDefault
                | TokenKind::KwDo
                | TokenKind::KwBridge
                | TokenKind::KwVm
                | TokenKind::KwInterop
                | TokenKind::KwVal
                | TokenKind::KwVar
                | TokenKind::KwConst
        )
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenKind::IntLiteral => "integer literal",
            TokenKind::FloatLiteral => "float literal",
            TokenKind::StringLiteral => "string literal",
            TokenKind::FStringStart => "f-string start",
            TokenKind::FStringMiddle => "f-string text",
            TokenKind::FStringEnd => "f-string end",
            TokenKind::TripleStringLiteral => "triple-quoted string",
            TokenKind::BoolTrue => "true",
            TokenKind::BoolFalse => "false",
            TokenKind::NullNone => "None",
            TokenKind::NullNull => "null",
            TokenKind::Identifier => "identifier",
            TokenKind::KwClass => "class",
            TokenKind::KwIf => "if",
            TokenKind::KwElse => "else",
            TokenKind::KwFor => "for",
            TokenKind::KwWhile => "while",
            TokenKind::KwReturn => "return",
            TokenKind::KwImport => "import",
            TokenKind::KwTry => "try",
            TokenKind::KwFinally => "finally",
            TokenKind::KwContinue => "continue",
            TokenKind::KwBreak => "break",
            TokenKind::KwAssert => "assert",
            TokenKind::KwDef => "def",
            TokenKind::KwElif => "elif",
            TokenKind::KwExcept => "except",
            TokenKind::KwRaise => "raise",
            TokenKind::KwPass => "pass",
            TokenKind::KwYield => "yield",
            TokenKind::KwAsync => "async",
            TokenKind::KwAwait => "await",
            TokenKind::KwWith => "with",
            TokenKind::KwAs => "as",
            TokenKind::KwFrom => "from",
            TokenKind::KwIn => "in",
            TokenKind::KwIs => "is",
            TokenKind::KwNot => "not",
            TokenKind::KwAnd => "and",
            TokenKind::KwOr => "or",
            TokenKind::KwLambda => "lambda",
            TokenKind::KwNonlocal => "nonlocal",
            TokenKind::KwGlobal => "global",
            TokenKind::KwDel => "del",
            TokenKind::KwMatch => "match",
            TokenKind::KwCase => "case",
            TokenKind::KwPublic => "public",
            TokenKind::KwPrivate => "private",
            TokenKind::KwProtected => "protected",
            TokenKind::KwStatic => "static",
            TokenKind::KwFinal => "final",
            TokenKind::KwAbstract => "abstract",
            TokenKind::KwInterface => "interface",
            TokenKind::KwEnum => "enum",
            TokenKind::KwExtends => "extends",
            TokenKind::KwImplements => "implements",
            TokenKind::KwNew => "new",
            TokenKind::KwThis => "this",
            TokenKind::KwSuper => "super",
            TokenKind::KwVoid => "void",
            TokenKind::KwThrows => "throws",
            TokenKind::KwThrow => "throw",
            TokenKind::KwCatch => "catch",
            TokenKind::KwSynchronized => "synchronized",
            TokenKind::KwVolatile => "volatile",
            TokenKind::KwTransient => "transient",
            TokenKind::KwNative => "native",
            TokenKind::KwInstanceof => "instanceof",
            TokenKind::KwSwitch => "switch",
            TokenKind::KwDefault => "default",
            TokenKind::KwDo => "do",
            TokenKind::KwBridge => "bridge",
            TokenKind::KwVm => "vm",
            TokenKind::KwInterop => "interop",
            TokenKind::KwVal => "val",
            TokenKind::KwVar => "var",
            TokenKind::KwConst => "const",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::DoubleSlash => "//",
            TokenKind::Percent => "%",
            TokenKind::DoubleStar => "**",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::LtEq => "<=",
            TokenKind::GtEq => ">=",
            TokenKind::AmpAmp => "&&",
            TokenKind::PipePipe => "||",
            TokenKind::Bang => "!",
            TokenKind::Amp => "&",
            TokenKind::Pipe => "|",
            TokenKind::Caret => "^",
            TokenKind::Tilde => "~",
            TokenKind::LShift => "<<",
            TokenKind::RShift => ">>",
            TokenKind::UnsignedRShift => ">>>",
            TokenKind::Eq => "=",
            TokenKind::PlusEq => "+=",
            TokenKind::MinusEq => "-=",
            TokenKind::StarEq => "*=",
            TokenKind::SlashEq => "/=",
            TokenKind::DoubleSlashEq => "//=",
            TokenKind::PercentEq => "%=",
            TokenKind::DoubleStarEq => "**=",
            TokenKind::AmpEq => "&=",
            TokenKind::PipeEq => "|=",
            TokenKind::CaretEq => "^=",
            TokenKind::LShiftEq => "<<=",
            TokenKind::RShiftEq => ">>=",
            TokenKind::ColonEq => ":=",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            TokenKind::DoubleColon => "::",
            TokenKind::Dot => ".",
            TokenKind::Ellipsis => "...",
            TokenKind::At => "@",
            TokenKind::Arrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::Question => "?",
            TokenKind::QuestionDot => "?.",
            TokenKind::DoubleQuestion => "??",
            TokenKind::Hash => "#",
            TokenKind::Indent => "INDENT",
            TokenKind::Dedent => "DEDENT",
            TokenKind::Newline => "NEWLINE",
            TokenKind::Eof => "EOF",
            TokenKind::Error => "ERROR",
        };
        write!(f, "{}", s)
    }
}
