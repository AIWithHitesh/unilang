// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Tests for the UniLang bytecode code generator.

use unilang_common::span::{SourceId, Span, Spanned};
use unilang_common::syntax_origin::SyntaxOrigin;
use unilang_parser::ast::*;

use crate::bytecode::{Opcode, Value};
use crate::compile;

// ── helpers ──────────────────────────────────────────────

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned::new(node, Span::new(0, 0))
}

fn make_module(stmts: Vec<Stmt>) -> Module {
    Module {
        source_id: SourceId(0),
        statements: stmts.into_iter().map(spanned).collect(),
    }
}

fn int_expr(v: i128) -> Expr {
    Expr::IntLit(v)
}

fn ident_expr(name: &str) -> Expr {
    Expr::Ident(name.to_string())
}

fn bool_expr(b: bool) -> Expr {
    Expr::BoolLit(b)
}

fn string_expr(s: &str) -> Expr {
    Expr::StringLit(s.to_string())
}

fn binary(lhs: Expr, op: BinOp, rhs: Expr) -> Expr {
    Expr::BinaryOp(Box::new(spanned(lhs)), op, Box::new(spanned(rhs)))
}

fn var_decl(name: &str, init: Option<Expr>) -> Stmt {
    Stmt::VarDecl(VarDecl {
        name: spanned(name.to_string()),
        type_ann: None,
        initializer: init.map(spanned),
        modifiers: Vec::new(),
        syntax: SyntaxOrigin::Python,
    })
}

fn empty_block() -> Block {
    Block {
        style: BlockStyle::Braces,
        statements: Vec::new(),
    }
}

fn block_with(stmts: Vec<Stmt>) -> Block {
    Block {
        style: BlockStyle::Braces,
        statements: stmts.into_iter().map(spanned).collect(),
    }
}

// ── tests ────────────────────────────────────────────────

#[test]
fn test_compile_integer_literal() {
    let module = make_module(vec![Stmt::Expr(int_expr(42))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Int(42)));
    // Expression statement should pop the value.
    assert_eq!(bc.instructions[1], Opcode::Pop);
}

#[test]
fn test_compile_binary_addition() {
    let module = make_module(vec![Stmt::Expr(binary(
        int_expr(1),
        BinOp::Add,
        int_expr(2),
    ))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Int(1)));
    assert_eq!(bc.instructions[1], Opcode::LoadConst(Value::Int(2)));
    assert_eq!(bc.instructions[2], Opcode::Add);
}

#[test]
fn test_compile_variable_declaration() {
    let module = make_module(vec![var_decl("x", Some(int_expr(10)))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Int(10)));
    assert_eq!(bc.instructions[1], Opcode::StoreLocal(0));
}

#[test]
fn test_compile_variable_reference() {
    let module = make_module(vec![
        var_decl("x", Some(int_expr(5))),
        Stmt::Expr(ident_expr("x")),
    ]);
    let bc = compile(&module).unwrap();
    // After StoreLocal(0), referencing x should LoadLocal(0).
    assert_eq!(bc.instructions[2], Opcode::LoadLocal(0));
}

#[test]
fn test_compile_if_statement() {
    let module = make_module(vec![Stmt::If(IfStmt {
        condition: spanned(bool_expr(true)),
        then_block: block_with(vec![Stmt::Expr(int_expr(1))]),
        elif_clauses: Vec::new(),
        else_block: None,
    })]);
    let bc = compile(&module).unwrap();
    // instruction 0: LoadConst(Bool(true))
    // instruction 1: JumpIfFalse(target)
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Bool(true)));
    match &bc.instructions[1] {
        Opcode::JumpIfFalse(target) => {
            // Target should skip the then-block (LoadConst + Pop = 2 instructions).
            assert_eq!(*target, 4);
        }
        other => panic!("Expected JumpIfFalse, got {:?}", other),
    }
}

#[test]
fn test_compile_while_loop() {
    let module = make_module(vec![Stmt::While(WhileStmt {
        condition: spanned(bool_expr(true)),
        body: block_with(vec![Stmt::Expr(int_expr(1))]),
    })]);
    let bc = compile(&module).unwrap();
    // 0: LoadConst(Bool(true))
    // 1: JumpIfFalse(end)
    // 2: LoadConst(Int(1))
    // 3: Pop
    // 4: Jump(0)  <-- back to condition
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Bool(true)));
    match &bc.instructions[1] {
        Opcode::JumpIfFalse(_) => {}
        other => panic!("Expected JumpIfFalse, got {:?}", other),
    }
    match &bc.instructions[4] {
        Opcode::Jump(target) => assert_eq!(*target, 0),
        other => panic!("Expected Jump(0), got {:?}", other),
    }
}

#[test]
fn test_compile_function_declaration() {
    let module = make_module(vec![Stmt::FunctionDecl(FunctionDecl {
        name: spanned("foo".to_string()),
        params: Vec::new(),
        return_type: None,
        body: block_with(vec![Stmt::Return(Some(spanned(int_expr(42))))]),
        visibility: Visibility::Default,
        modifiers: Vec::new(),
        decorators: Vec::new(),
        is_async: false,
        syntax: SyntaxOrigin::Python,
    })]);
    let bc = compile(&module).unwrap();
    // Should have a MakeFunction instruction.
    assert!(bc
        .instructions
        .iter()
        .any(|op| matches!(op, Opcode::MakeFunction(0))));
    // Function table should have one entry.
    assert_eq!(bc.functions.len(), 1);
    assert_eq!(bc.functions[0].name, "foo");
    // Function body should end with Return.
    assert!(bc.functions[0]
        .code
        .iter()
        .any(|op| matches!(op, Opcode::Return)));
}

#[test]
fn test_compile_function_call() {
    // First declare a function, then call it.
    let module = make_module(vec![
        Stmt::FunctionDecl(FunctionDecl {
            name: spanned("bar".to_string()),
            params: Vec::new(),
            return_type: None,
            body: block_with(vec![Stmt::Return(Some(spanned(int_expr(1))))]),
            visibility: Visibility::Default,
            modifiers: Vec::new(),
            decorators: Vec::new(),
            is_async: false,
            syntax: SyntaxOrigin::Python,
        }),
        Stmt::Expr(Expr::Call(Box::new(spanned(ident_expr("bar"))), Vec::new())),
    ]);
    let bc = compile(&module).unwrap();
    // Should contain a Call(0) instruction (0 args).
    assert!(bc
        .instructions
        .iter()
        .any(|op| matches!(op, Opcode::Call(0))));
}

#[test]
fn test_compile_print_statement() {
    let module = make_module(vec![Stmt::Expr(Expr::Call(
        Box::new(spanned(ident_expr("print"))),
        vec![Argument {
            name: None,
            value: spanned(string_expr("hello")),
        }],
    ))]);
    let bc = compile(&module).unwrap();
    // Should have a Print instruction.
    assert!(bc.instructions.iter().any(|op| matches!(op, Opcode::Print)));
}

#[test]
fn test_compile_return_statement() {
    let module = make_module(vec![Stmt::FunctionDecl(FunctionDecl {
        name: spanned("get_val".to_string()),
        params: Vec::new(),
        return_type: None,
        body: block_with(vec![Stmt::Return(Some(spanned(int_expr(99))))]),
        visibility: Visibility::Default,
        modifiers: Vec::new(),
        decorators: Vec::new(),
        is_async: false,
        syntax: SyntaxOrigin::Python,
    })]);
    let bc = compile(&module).unwrap();
    let func = &bc.functions[0];
    // Body should contain LoadConst(99) followed by Return.
    assert_eq!(func.code[0], Opcode::LoadConst(Value::Int(99)));
    assert_eq!(func.code[1], Opcode::Return);
}

#[test]
fn test_compile_string_literal() {
    let module = make_module(vec![Stmt::Expr(string_expr("world"))]);
    let bc = compile(&module).unwrap();
    assert_eq!(
        bc.instructions[0],
        Opcode::LoadConst(Value::String("world".to_string()))
    );
}

#[test]
fn test_compile_bool_literal() {
    let module = make_module(vec![Stmt::Expr(bool_expr(false))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Bool(false)));
}

#[test]
fn test_compile_unary_negation() {
    let module = make_module(vec![Stmt::Expr(Expr::UnaryOp(
        UnaryOp::Neg,
        Box::new(spanned(int_expr(5))),
    ))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Int(5)));
    assert_eq!(bc.instructions[1], Opcode::Neg);
}

#[test]
fn test_compile_list_literal() {
    let module = make_module(vec![Stmt::Expr(Expr::List(vec![
        spanned(int_expr(1)),
        spanned(int_expr(2)),
        spanned(int_expr(3)),
    ]))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions[0], Opcode::LoadConst(Value::Int(1)));
    assert_eq!(bc.instructions[1], Opcode::LoadConst(Value::Int(2)));
    assert_eq!(bc.instructions[2], Opcode::LoadConst(Value::Int(3)));
    assert_eq!(bc.instructions[3], Opcode::MakeList(3));
}

#[test]
fn test_compile_if_else() {
    let module = make_module(vec![Stmt::If(IfStmt {
        condition: spanned(bool_expr(true)),
        then_block: block_with(vec![Stmt::Expr(int_expr(1))]),
        elif_clauses: Vec::new(),
        else_block: Some(block_with(vec![Stmt::Expr(int_expr(2))])),
    })]);
    let bc = compile(&module).unwrap();
    // Should contain both JumpIfFalse and Jump (for skipping else).
    let has_jump_if_false = bc
        .instructions
        .iter()
        .any(|op| matches!(op, Opcode::JumpIfFalse(_)));
    let has_jump = bc
        .instructions
        .iter()
        .any(|op| matches!(op, Opcode::Jump(_)));
    assert!(has_jump_if_false);
    assert!(has_jump);
}

#[test]
fn test_bytecode_ends_with_halt() {
    let module = make_module(vec![Stmt::Expr(int_expr(1))]);
    let bc = compile(&module).unwrap();
    assert_eq!(bc.instructions.last(), Some(&Opcode::Halt));
}
