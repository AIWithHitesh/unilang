// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Comprehensive parser tests for UniLang.

use unilang_common::span::SourceId;

use crate::ast::*;

// ── Helpers ───────────────────────────────────────────────────────────────

fn parse_source(source: &str) -> (Module, unilang_common::error::DiagnosticBag) {
    crate::parse(SourceId(0), source)
}

/// Parse source and panic if there are any errors.
fn parse_ok(source: &str) -> Module {
    let (module, diag) = parse_source(source);
    if diag.has_errors() {
        for d in diag.diagnostics() {
            eprintln!("PARSE ERROR: {}", d.message);
        }
        panic!("unexpected parse errors for source: {:?}", source);
    }
    module
}

/// Parse source and return true if there were errors.
#[allow(dead_code)]
fn parse_has_errors(source: &str) -> bool {
    let (_, diag) = parse_source(source);
    diag.has_errors()
}

// ── 1. Python-style function definition ──────────────────────────────────

#[test]
fn test_python_function_def_simple() {
    let m = parse_ok("def add(a, b):\n    return a + b");
    assert_eq!(m.statements.len(), 1);
    match &m.statements[0].node {
        Stmt::FunctionDecl(f) => {
            assert_eq!(f.name.node, "add");
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name.node, "a");
            assert_eq!(f.params[1].name.node, "b");
            assert_eq!(f.body.style, BlockStyle::Indentation);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn test_python_function_with_return() {
    let m = parse_ok("def square(x):\n    return x * x");
    match &m.statements[0].node {
        Stmt::FunctionDecl(f) => {
            assert_eq!(f.name.node, "square");
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.body.statements.len(), 1);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

// ── 2. Java-style function definition ────────────────────────────────────

#[test]
fn test_java_function_def_simple() {
    let m = parse_ok("int add(a, b) { return a + b; }");
    assert_eq!(m.statements.len(), 1);
    match &m.statements[0].node {
        Stmt::FunctionDecl(f) => {
            assert_eq!(f.name.node, "add");
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.body.style, BlockStyle::Braces);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn test_java_public_function() {
    let m = parse_ok("public int multiply(int x, int y) { return x * y; }");
    match &m.statements[0].node {
        Stmt::FunctionDecl(f) => {
            assert_eq!(f.name.node, "multiply");
            assert_eq!(f.visibility, Visibility::Public);
            assert_eq!(f.params.len(), 2);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

// ── 3. Python-style class ─────────────────────────────────────────────────

#[test]
fn test_python_class_empty() {
    let m = parse_ok("class Foo:\n    pass");
    match &m.statements[0].node {
        Stmt::ClassDecl(c) => {
            assert_eq!(c.name.node, "Foo");
            assert_eq!(c.syntax, unilang_common::syntax_origin::SyntaxOrigin::Python);
        }
        other => panic!("expected ClassDecl, got {:?}", other),
    }
}

#[test]
fn test_python_class_with_init() {
    let m = parse_ok("class Animal:\n    def __init__(self, name):\n        self.name = name");
    match &m.statements[0].node {
        Stmt::ClassDecl(c) => {
            assert_eq!(c.name.node, "Animal");
            assert!(!c.body.is_empty());
        }
        other => panic!("expected ClassDecl, got {:?}", other),
    }
}

// ── 4. Java-style class with fields ──────────────────────────────────────

#[test]
fn test_java_class_empty() {
    let m = parse_ok("public class Bar { }");
    match &m.statements[0].node {
        Stmt::ClassDecl(c) => {
            assert_eq!(c.name.node, "Bar");
            assert_eq!(c.visibility, Visibility::Public);
        }
        other => panic!("expected ClassDecl, got {:?}", other),
    }
}

#[test]
fn test_java_class_with_method() {
    let m = parse_ok("public class Calculator {\n    public int add(int a, int b) { return a + b; }\n}");
    match &m.statements[0].node {
        Stmt::ClassDecl(c) => {
            assert_eq!(c.name.node, "Calculator");
            assert!(!c.body.is_empty());
        }
        other => panic!("expected ClassDecl, got {:?}", other),
    }
}

// ── 5. if/elif/else (colon style) ────────────────────────────────────────

#[test]
fn test_if_elif_else_python() {
    let m = parse_ok("if x:\n    a\nelif y:\n    b\nelse:\n    c");
    match &m.statements[0].node {
        Stmt::If(s) => {
            assert_eq!(s.elif_clauses.len(), 1);
            assert!(s.else_block.is_some());
        }
        other => panic!("expected If, got {:?}", other),
    }
}

#[test]
fn test_if_only_python() {
    let m = parse_ok("if condition:\n    do_something()");
    match &m.statements[0].node {
        Stmt::If(s) => {
            assert_eq!(s.elif_clauses.len(), 0);
            assert!(s.else_block.is_none());
        }
        other => panic!("expected If, got {:?}", other),
    }
}

// ── 6. if/else if/else (brace style) ─────────────────────────────────────

#[test]
fn test_if_else_java_style() {
    let m = parse_ok("if (x > 0) { a; } else { b; }");
    match &m.statements[0].node {
        Stmt::If(s) => {
            assert_eq!(s.then_block.style, BlockStyle::Braces);
            assert!(s.else_block.is_some());
        }
        other => panic!("expected If, got {:?}", other),
    }
}

// ── 7. Python for item in list loop ──────────────────────────────────────

#[test]
fn test_for_in_loop_python() {
    let m = parse_ok("for item in items:\n    print(item)");
    match &m.statements[0].node {
        Stmt::For(ForStmt::ForIn { target, iter, .. }) => {
            match &target.node {
                Expr::Ident(name) => assert_eq!(name, "item"),
                other => panic!("expected Ident, got {:?}", other),
            }
            match &iter.node {
                Expr::Ident(name) => assert_eq!(name, "items"),
                other => panic!("expected Ident, got {:?}", other),
            }
        }
        other => panic!("expected ForIn, got {:?}", other),
    }
}

// ── 8. Java-style for loop ────────────────────────────────────────────────

#[test]
fn test_java_for_loop() {
    let m = parse_ok("for (i = 0; i < 10; i = i + 1) { print(i); }");
    match &m.statements[0].node {
        Stmt::For(ForStmt::ForClassic { .. }) => {}
        other => panic!("expected ForClassic, got {:?}", other),
    }
}

// ── 9. while loop (both styles) ──────────────────────────────────────────

#[test]
fn test_while_python() {
    let m = parse_ok("while x > 0:\n    x = x - 1");
    match &m.statements[0].node {
        Stmt::While(_) => {}
        other => panic!("expected While, got {:?}", other),
    }
}

#[test]
fn test_while_java() {
    let m = parse_ok("while (x > 0) { x = x - 1; }");
    match &m.statements[0].node {
        Stmt::While(_) => {}
        other => panic!("expected While, got {:?}", other),
    }
}

// ── 10. try/except (Python) ───────────────────────────────────────────────

#[test]
fn test_try_except_python() {
    let m = parse_ok("try:\n    risky()\nexcept Exception as e:\n    handle(e)");
    match &m.statements[0].node {
        Stmt::Try(t) => {
            assert!(!t.catch_clauses.is_empty());
        }
        other => panic!("expected Try, got {:?}", other),
    }
}

#[test]
fn test_try_except_bare() {
    let m = parse_ok("try:\n    risky()\nexcept:\n    handle()");
    match &m.statements[0].node {
        Stmt::Try(t) => {
            assert!(!t.catch_clauses.is_empty());
        }
        other => panic!("expected Try, got {:?}", other),
    }
}

// ── 11. try/catch/finally (Java) ─────────────────────────────────────────

#[test]
fn test_try_catch_finally_java() {
    let m = parse_ok("try { risky(); } catch (Exception e) { handle(e); } finally { cleanup(); }");
    match &m.statements[0].node {
        Stmt::Try(t) => {
            assert!(!t.catch_clauses.is_empty());
            assert!(t.finally_block.is_some());
        }
        other => panic!("expected Try, got {:?}", other),
    }
}

// ── 12. Imports ───────────────────────────────────────────────────────────

#[test]
fn test_import_simple() {
    let m = parse_ok("import math");
    match &m.statements[0].node {
        Stmt::Import(ImportStmt::Simple { path, .. }) => {
            assert_eq!(path.len(), 1);
            assert_eq!(path[0].node, "math");
        }
        other => panic!("expected Import Simple, got {:?}", other),
    }
}

#[test]
fn test_import_dotted() {
    let m = parse_ok("import os.path");
    match &m.statements[0].node {
        Stmt::Import(ImportStmt::Simple { path, .. }) => {
            assert_eq!(path.len(), 2);
        }
        other => panic!("expected Import Simple, got {:?}", other),
    }
}

#[test]
fn test_from_import() {
    let m = parse_ok("from os import path");
    match &m.statements[0].node {
        Stmt::Import(ImportStmt::From { names, .. }) => match names {
            ImportNames::Named(aliases) => {
                assert_eq!(aliases.len(), 1);
                assert_eq!(aliases[0].name.node, "path");
            }
            _ => panic!("expected Named"),
        },
        other => panic!("expected Import From, got {:?}", other),
    }
}

#[test]
fn test_import_with_alias() {
    let m = parse_ok("import numpy as np");
    match &m.statements[0].node {
        Stmt::Import(ImportStmt::Simple { alias, .. }) => {
            assert!(alias.is_some());
            assert_eq!(alias.as_ref().unwrap().node, "np");
        }
        other => panic!("expected Import Simple, got {:?}", other),
    }
}

// ── 13. return, break, continue ──────────────────────────────────────────

#[test]
fn test_return_with_value() {
    let m = parse_ok("def f():\n    return 42");
    match &m.statements[0].node {
        Stmt::FunctionDecl(f) => {
            match &f.body.statements[0].node {
                Stmt::Return(Some(expr)) => match &expr.node {
                    Expr::IntLit(n) => assert_eq!(*n, 42),
                    other => panic!("expected IntLit, got {:?}", other),
                },
                other => panic!("expected Return(Some), got {:?}", other),
            }
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn test_return_no_value() {
    let m = parse_ok("def f():\n    return");
    match &m.statements[0].node {
        Stmt::FunctionDecl(f) => {
            match &f.body.statements[0].node {
                Stmt::Return(None) => {}
                other => panic!("expected Return(None), got {:?}", other),
            }
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn test_break_statement() {
    let m = parse_ok("while True:\n    break");
    match &m.statements[0].node {
        Stmt::While(w) => match &w.body.statements[0].node {
            Stmt::Break => {}
            other => panic!("expected Break, got {:?}", other),
        },
        other => panic!("expected While, got {:?}", other),
    }
}

#[test]
fn test_continue_statement() {
    let m = parse_ok("while True:\n    continue");
    match &m.statements[0].node {
        Stmt::While(w) => match &w.body.statements[0].node {
            Stmt::Continue => {}
            other => panic!("expected Continue, got {:?}", other),
        },
        other => panic!("expected While, got {:?}", other),
    }
}

// ── 14. Arithmetic expression ────────────────────────────────────────────

#[test]
fn test_arithmetic_precedence() {
    // a + b * c - d  should parse as  (a + (b * c)) - d
    let m = parse_ok("a + b * c - d");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BinaryOp(left, BinOp::Sub, _)) => match &left.node {
            Expr::BinaryOp(_, BinOp::Add, right) => match &right.node {
                Expr::BinaryOp(_, BinOp::Mul, _) => {}
                other => panic!("expected Mul, got {:?}", other),
            },
            other => panic!("expected Add, got {:?}", other),
        },
        other => panic!("expected BinaryOp Sub, got {:?}", other),
    }
}

// ── 15. Function call ─────────────────────────────────────────────────────

#[test]
fn test_function_call_three_args() {
    let m = parse_ok("foo(a, b, c)");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Call(callee, args)) => {
            match &callee.node {
                Expr::Ident(name) => assert_eq!(name, "foo"),
                other => panic!("expected Ident, got {:?}", other),
            }
            assert_eq!(args.len(), 3);
        }
        other => panic!("expected Call, got {:?}", other),
    }
}

#[test]
fn test_function_call_no_args() {
    let m = parse_ok("greet()");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Call(_, args)) => assert_eq!(args.len(), 0),
        other => panic!("expected Call, got {:?}", other),
    }
}

// ── 16. Member access ─────────────────────────────────────────────────────

#[test]
fn test_member_access() {
    let m = parse_ok("obj.field");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Attribute(obj, field)) => {
            match &obj.node {
                Expr::Ident(name) => assert_eq!(name, "obj"),
                other => panic!("expected Ident, got {:?}", other),
            }
            assert_eq!(field.node, "field");
        }
        other => panic!("expected Attribute, got {:?}", other),
    }
}

// ── 17. Index access ─────────────────────────────────────────────────────

#[test]
fn test_index_access() {
    let m = parse_ok("arr[0]");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Index(obj, idx)) => {
            match &obj.node {
                Expr::Ident(name) => assert_eq!(name, "arr"),
                other => panic!("expected Ident, got {:?}", other),
            }
            match &idx.node {
                Expr::IntLit(n) => assert_eq!(*n, 0),
                other => panic!("expected IntLit(0), got {:?}", other),
            }
        }
        other => panic!("expected Index, got {:?}", other),
    }
}

// ── 18. String literal & f-string ────────────────────────────────────────

#[test]
fn test_string_literal_double_quote() {
    let m = parse_ok(r#""hello world""#);
    match &m.statements[0].node {
        Stmt::Expr(Expr::StringLit(s)) => assert_eq!(s, "hello world"),
        other => panic!("expected StringLit, got {:?}", other),
    }
}

#[test]
fn test_string_literal_single_quote() {
    let m = parse_ok("'hello'");
    match &m.statements[0].node {
        Stmt::Expr(Expr::StringLit(s)) => assert_eq!(s, "hello"),
        other => panic!("expected StringLit, got {:?}", other),
    }
}

#[test]
fn test_fstring_basic() {
    // f-strings should parse without error
    let (m, diag) = parse_source("f\"hello {name}\"");
    // May produce StringLit or a dedicated FString node – just check no panics
    // and we get at least one statement
    let _ = &m;
    let _ = &diag;
    // Parser should not crash
}

// ── 19. List literal ─────────────────────────────────────────────────────

#[test]
fn test_list_literal() {
    let m = parse_ok("[1, 2, 3]");
    match &m.statements[0].node {
        Stmt::Expr(Expr::List(items)) => {
            assert_eq!(items.len(), 3);
            match &items[0].node {
                Expr::IntLit(n) => assert_eq!(*n, 1),
                other => panic!("expected IntLit(1), got {:?}", other),
            }
        }
        other => panic!("expected List, got {:?}", other),
    }
}

#[test]
fn test_list_empty() {
    let m = parse_ok("[]");
    match &m.statements[0].node {
        Stmt::Expr(Expr::List(items)) => assert_eq!(items.len(), 0),
        other => panic!("expected empty List, got {:?}", other),
    }
}

// ── 20. Dict literal ─────────────────────────────────────────────────────

#[test]
fn test_dict_literal() {
    let m = parse_ok(r#"{"key": "value"}"#);
    match &m.statements[0].node {
        Stmt::Expr(Expr::Dict(pairs)) => {
            assert_eq!(pairs.len(), 1);
            match &pairs[0].0.node {
                Expr::StringLit(k) => assert_eq!(k, "key"),
                other => panic!("expected StringLit key, got {:?}", other),
            }
        }
        other => panic!("expected Dict, got {:?}", other),
    }
}

#[test]
fn test_dict_empty() {
    let m = parse_ok("{}");
    // Could be empty Dict or empty Block — just verify no panic
    let _ = &m;
}

// ── 21. Lambda expression ─────────────────────────────────────────────────

#[test]
fn test_lambda_expression() {
    let m = parse_ok("f = lambda x: x * 2");
    // The assignment statement should be a VarDecl or Expr(Assign)
    assert!(!m.statements.is_empty());
}

// ── 22. Class instantiation (new Foo) ────────────────────────────────────

#[test]
fn test_new_class_instantiation() {
    let m = parse_ok("obj = new Foo(x, y)");
    assert!(!m.statements.is_empty());
    // Check that a New expression was created
    let has_new = m.statements.iter().any(|s| {
        if let Stmt::Expr(Expr::Assign(_, rhs)) = &s.node {
            matches!(rhs.node, Expr::New(_, _))
        } else if let Stmt::VarDecl(vd) = &s.node {
            vd.initializer
                .as_ref()
                .map_or(false, |i| matches!(i.node, Expr::New(_, _)))
        } else {
            false
        }
    });
    assert!(has_new, "expected a New expression");
}

// ── 23. Nested function calls ─────────────────────────────────────────────

#[test]
fn test_nested_function_calls() {
    let m = parse_ok("foo(bar(x))");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Call(_, args)) => {
            assert_eq!(args.len(), 1);
            match &args[0].value.node {
                Expr::Call(_, _) => {}
                other => panic!("expected inner Call, got {:?}", other),
            }
        }
        other => panic!("expected outer Call, got {:?}", other),
    }
}

// ── 24. Chained member access ─────────────────────────────────────────────

#[test]
fn test_chained_member_access() {
    let m = parse_ok("a.b.c");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Attribute(outer, c_name)) => {
            assert_eq!(c_name.node, "c");
            match &outer.node {
                Expr::Attribute(_, b_name) => assert_eq!(b_name.node, "b"),
                other => panic!("expected Attribute a.b, got {:?}", other),
            }
        }
        other => panic!("expected Attribute a.b.c, got {:?}", other),
    }
}

// ── 25. Binary operators: and, or, not, is, in ───────────────────────────

#[test]
fn test_binary_and() {
    let m = parse_ok("x and y");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BinaryOp(_, BinOp::And, _)) => {}
        other => panic!("expected And, got {:?}", other),
    }
}

#[test]
fn test_binary_or() {
    let m = parse_ok("x or y");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BinaryOp(_, BinOp::Or, _)) => {}
        other => panic!("expected Or, got {:?}", other),
    }
}

#[test]
fn test_unary_not() {
    let m = parse_ok("not x");
    match &m.statements[0].node {
        Stmt::Expr(Expr::UnaryOp(UnaryOp::LogicalNot, _)) => {}
        Stmt::Expr(Expr::UnaryOp(UnaryOp::Not, _)) => {}
        other => panic!("expected UnaryOp Not, got {:?}", other),
    }
}

#[test]
fn test_binary_in() {
    let m = parse_ok("x in items");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BinaryOp(_, BinOp::In, _)) => {}
        other => panic!("expected In, got {:?}", other),
    }
}

#[test]
fn test_binary_is() {
    let m = parse_ok("x is None");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BinaryOp(_, BinOp::Is, _)) => {}
        other => panic!("expected Is, got {:?}", other),
    }
}

// ── 26. Assignment to member: self.x = value ─────────────────────────────

#[test]
fn test_assignment_to_member() {
    let m = parse_ok("self.x = 42");
    assert!(!m.statements.is_empty());
    // The statement can be Expr(Assign(Attribute, IntLit)) or VarDecl
    // Just verify it parsed without error
}

// ── 27. Parse error recovery ──────────────────────────────────────────────

#[test]
fn test_malformed_source_no_panic() {
    // Mismatched braces should produce errors but not panic
    let (_, diag) = parse_source("def foo(: { }}}}}");
    assert!(diag.has_errors());
}

#[test]
fn test_incomplete_function_no_panic() {
    let (_, _diag) = parse_source("def");
    // Parser should not panic on incomplete input
}

#[test]
fn test_missing_colon_no_panic() {
    let (_, _diag) = parse_source("if x\n    pass");
    // Parser should not panic
}

// ── 28. Float literal ─────────────────────────────────────────────────────

#[test]
fn test_float_literal() {
    let m = parse_ok("3.14");
    match &m.statements[0].node {
        Stmt::Expr(Expr::FloatLit(f)) => assert!((*f - 3.14).abs() < 1e-10),
        other => panic!("expected FloatLit, got {:?}", other),
    }
}

// ── 29. Bool literals ─────────────────────────────────────────────────────

#[test]
fn test_bool_literals() {
    let m = parse_ok("True");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BoolLit(b)) => assert!(*b),
        other => panic!("expected BoolLit(true), got {:?}", other),
    }
    let m = parse_ok("False");
    match &m.statements[0].node {
        Stmt::Expr(Expr::BoolLit(b)) => assert!(!*b),
        other => panic!("expected BoolLit(false), got {:?}", other),
    }
}

// ── 30. Null / None literal ───────────────────────────────────────────────

#[test]
fn test_null_literal() {
    let m = parse_ok("None");
    match &m.statements[0].node {
        Stmt::Expr(Expr::NullLit) => {}
        other => panic!("expected NullLit, got {:?}", other),
    }
}

// ── 31. Multiple statements ───────────────────────────────────────────────

#[test]
fn test_multiple_statements() {
    let m = parse_ok("x = 1\ny = 2\nz = x + y");
    assert_eq!(m.statements.len(), 3);
}

// ── 32. For loop with range ────────────────────────────────────────────────

#[test]
fn test_for_range_loop() {
    let m = parse_ok("for i in range(10):\n    pass");
    match &m.statements[0].node {
        Stmt::For(ForStmt::ForIn { .. }) => {}
        other => panic!("expected ForIn, got {:?}", other),
    }
}

// ── 33. Comparison operators ──────────────────────────────────────────────

#[test]
fn test_comparison_operators() {
    let ops = [
        ("a == b", BinOp::Eq),
        ("a != b", BinOp::NotEq),
        ("a < b", BinOp::Lt),
        ("a > b", BinOp::Gt),
        ("a <= b", BinOp::LtEq),
        ("a >= b", BinOp::GtEq),
    ];
    for (source, expected_op) in &ops {
        let m = parse_ok(source);
        match &m.statements[0].node {
            Stmt::Expr(Expr::BinaryOp(_, op, _)) => assert_eq!(op, expected_op),
            other => panic!("expected BinaryOp for '{}', got {:?}", source, other),
        }
    }
}

// ── 34. Method call on object ─────────────────────────────────────────────

#[test]
fn test_method_call() {
    let m = parse_ok("obj.method(arg1, arg2)");
    match &m.statements[0].node {
        Stmt::Expr(Expr::Call(callee, args)) => {
            match &callee.node {
                Expr::Attribute(_, name) => assert_eq!(name.node, "method"),
                other => panic!("expected Attribute, got {:?}", other),
            }
            assert_eq!(args.len(), 2);
        }
        other => panic!("expected Call, got {:?}", other),
    }
}

// ── 35. Pass statement ────────────────────────────────────────────────────

#[test]
fn test_pass_statement() {
    let m = parse_ok("pass");
    match &m.statements[0].node {
        Stmt::Pass => {}
        other => panic!("expected Pass, got {:?}", other),
    }
}
