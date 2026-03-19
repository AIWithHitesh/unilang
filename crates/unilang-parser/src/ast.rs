// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang unified AST node definitions.
//!
//! These types represent both Python-style and Java-style constructs
//! in a single tree structure.

use unilang_common::span::{SourceId, Spanned};
use unilang_common::syntax_origin::SyntaxOrigin;

/// A parsed UniLang module (one `.uniL` source file).
#[derive(Debug, Clone)]
pub struct Module {
    pub source_id: SourceId,
    pub statements: Vec<Spanned<Stmt>>,
}

// ── Statements ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// An expression used as a statement.
    Expr(Expr),
    /// Variable declaration (Java-style or Python-style).
    VarDecl(VarDecl),
    /// Function or method declaration.
    FunctionDecl(FunctionDecl),
    /// Class declaration.
    ClassDecl(ClassDecl),
    /// Import statement (any style).
    Import(ImportStmt),
    /// `if` / `elif` / `else if` / `else`
    If(IfStmt),
    /// `for` loop (Python or Java style).
    For(ForStmt),
    /// `while` loop.
    While(WhileStmt),
    /// `do ... while` (Java only).
    DoWhile(DoWhileStmt),
    /// `try` / `except` / `catch` / `finally`
    Try(TryStmt),
    /// `with` statement (Python).
    With(WithStmt),
    /// `return [expr]`
    Return(Option<Spanned<Expr>>),
    /// `raise expr` (Python) or `throw expr` (Java).
    Throw(Spanned<Expr>),
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `pass`
    Pass,
    /// `assert expr [, msg]`
    Assert(Spanned<Expr>, Option<Spanned<Expr>>),
    /// A block (braces or indentation).
    Block(Block),
    /// Error recovery placeholder.
    Error,
}

// ── Declarations ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: Spanned<String>,
    pub type_ann: Option<Spanned<TypeExpr>>,
    pub initializer: Option<Spanned<Expr>>,
    pub modifiers: Vec<Modifier>,
    pub syntax: SyntaxOrigin,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub body: Block,
    pub visibility: Visibility,
    pub modifiers: Vec<Modifier>,
    pub decorators: Vec<Spanned<Decorator>>,
    pub is_async: bool,
    pub syntax: SyntaxOrigin,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: Spanned<String>,
    pub type_params: Vec<Spanned<TypeExpr>>,
    pub bases: Vec<Spanned<TypeExpr>>,
    pub extends: Option<Spanned<TypeExpr>>,
    pub implements: Vec<Spanned<TypeExpr>>,
    pub body: Vec<Spanned<Stmt>>,
    pub visibility: Visibility,
    pub modifiers: Vec<Modifier>,
    pub decorators: Vec<Spanned<Decorator>>,
    pub syntax: SyntaxOrigin,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Spanned<String>,
    pub type_ann: Option<Spanned<TypeExpr>>,
    pub default: Option<Spanned<Expr>>,
    pub is_vararg: bool,
    pub is_kwarg: bool,
}

#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: Spanned<String>,
    pub args: Vec<Spanned<Expr>>,
}

// ── Blocks ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Block {
    pub style: BlockStyle,
    pub statements: Vec<Spanned<Stmt>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStyle {
    Braces,
    Indentation,
    Hybrid,
}

// ── Visibility & Modifiers ────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Static,
    Final,
    Abstract,
    Native,
    Synchronized,
    Volatile,
    Transient,
}

// ── Expressions ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    IntLit(i128),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    NullLit,

    // Identifiers and access
    Ident(String),
    Attribute(Box<Spanned<Expr>>, Spanned<String>),
    Index(Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    // Operators
    BinaryOp(Box<Spanned<Expr>>, BinOp, Box<Spanned<Expr>>),
    UnaryOp(UnaryOp, Box<Spanned<Expr>>),

    // Calls
    Call(Box<Spanned<Expr>>, Vec<Argument>),
    New(Spanned<TypeExpr>, Vec<Argument>),

    // Lambda
    Lambda(Vec<Param>, Box<Spanned<Expr>>),

    // Ternary
    Ternary {
        condition: Box<Spanned<Expr>>,
        then_expr: Box<Spanned<Expr>>,
        else_expr: Box<Spanned<Expr>>,
    },

    // Collections
    List(Vec<Spanned<Expr>>),
    Dict(Vec<(Spanned<Expr>, Spanned<Expr>)>),
    Set(Vec<Spanned<Expr>>),

    // Comprehension
    ListComp {
        element: Box<Spanned<Expr>>,
        clauses: Vec<CompClause>,
    },

    // Assignment expressions
    Assign(Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    // Cast
    Cast(Spanned<TypeExpr>, Box<Spanned<Expr>>),

    // Await
    Await(Box<Spanned<Expr>>),

    // Error recovery
    Error,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Option<Spanned<String>>,
    pub value: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct CompClause {
    pub target: Spanned<Expr>,
    pub iter: Spanned<Expr>,
    pub conditions: Vec<Spanned<Expr>>,
}

// ── Operators ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    UnsignedRShift,
    In,
    NotIn,
    Is,
    IsNot,
    Instanceof,
    NullCoalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Pos,
    Not,
    BitNot,
    LogicalNot,
}

// ── Type Expressions ──────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String),
    Qualified(Vec<String>),
    Generic(Box<Spanned<TypeExpr>>, Vec<Spanned<TypeExpr>>),
    Array(Box<Spanned<TypeExpr>>),
    Optional(Box<Spanned<TypeExpr>>),
    Union(Vec<Spanned<TypeExpr>>),
    Tuple(Vec<Spanned<TypeExpr>>),
    Inferred,
}

// ── Control Flow ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Spanned<Expr>,
    pub then_block: Block,
    pub elif_clauses: Vec<(Spanned<Expr>, Block)>,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub enum ForStmt {
    /// Python-style: `for x in iter:`
    ForIn {
        target: Spanned<Expr>,
        iter: Spanned<Expr>,
        body: Block,
    },
    /// Java-style: `for (init; cond; update)`
    ForClassic {
        init: Option<Box<Spanned<Stmt>>>,
        condition: Option<Spanned<Expr>>,
        update: Option<Spanned<Expr>>,
        body: Block,
    },
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Spanned<Expr>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct DoWhileStmt {
    pub body: Block,
    pub condition: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct TryStmt {
    pub body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub finally_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub exception_type: Option<Spanned<TypeExpr>>,
    pub name: Option<Spanned<String>>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct WithStmt {
    pub items: Vec<WithItem>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct WithItem {
    pub context: Spanned<Expr>,
    pub alias: Option<Spanned<String>>,
}

// ── Imports ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ImportStmt {
    /// `import foo.bar` or `import foo.bar as baz`
    Simple {
        path: Vec<Spanned<String>>,
        alias: Option<Spanned<String>>,
    },
    /// `from foo.bar import x, y` or `from foo.bar import *`
    From {
        path: Vec<Spanned<String>>,
        names: ImportNames,
    },
    /// `import static foo.bar.Baz`
    Static { path: Vec<Spanned<String>> },
}

#[derive(Debug, Clone)]
pub enum ImportNames {
    /// `import x, y as z`
    Named(Vec<ImportAlias>),
    /// `import *`
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct ImportAlias {
    pub name: Spanned<String>,
    pub alias: Option<Spanned<String>>,
}
