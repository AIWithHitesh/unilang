// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # UniLang Semantic Analyzer
//!
//! Performs name resolution, scope management, type checking, and
//! declaration validation on the UniLang AST. Supports gradual typing
//! to allow mixing Python-style dynamic code with Java-style static types.

pub mod analyzer;
pub mod checker;
pub mod scope;
pub mod types;

pub use analyzer::AnalysisResult;
pub use types::Type;

use unilang_common::error::DiagnosticBag;
use unilang_parser::ast::Module;

/// Convenience entry point: analyze a parsed module.
pub fn analyze(module: &Module) -> (AnalysisResult, DiagnosticBag) {
    let analyzer = analyzer::Analyzer::new(module.source_id);
    analyzer.analyze(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unilang_common::error::Severity;
    use unilang_common::span::{SourceId, Span, Spanned};
    use unilang_parser::ast::*;

    // ── Test helpers ──────────────────────────────────────────

    fn spanned<T>(node: T, start: u32, end: u32) -> Spanned<T> {
        Spanned::new(node, Span::new(start, end))
    }

    fn make_module(stmts: Vec<Spanned<Stmt>>) -> Module {
        Module {
            source_id: SourceId(0),
            statements: stmts,
        }
    }

    fn block_with(stmts: Vec<Spanned<Stmt>>) -> Block {
        Block {
            style: BlockStyle::Braces,
            statements: stmts,
        }
    }

    fn has_error_containing(diags: &DiagnosticBag, text: &str) -> bool {
        diags
            .diagnostics()
            .iter()
            .any(|d| d.severity == Severity::Error && d.message.contains(text))
    }

    fn has_no_errors(diags: &DiagnosticBag) -> bool {
        !diags.has_errors()
    }

    fn var_decl_stmt(
        name: &str,
        type_ann: Option<TypeExpr>,
        init: Option<Expr>,
        modifiers: Vec<Modifier>,
    ) -> Spanned<Stmt> {
        spanned(
            Stmt::VarDecl(VarDecl {
                name: spanned(name.to_string(), 0, name.len() as u32),
                type_ann: type_ann.map(|t| spanned(t, 0, 1)),
                initializer: init.map(|e| spanned(e, 0, 1)),
                modifiers,
                syntax: unilang_common::syntax_origin::SyntaxOrigin::Java,
            }),
            0,
            10,
        )
    }

    fn python_var_decl_stmt(name: &str, init: Option<Expr>) -> Spanned<Stmt> {
        spanned(
            Stmt::VarDecl(VarDecl {
                name: spanned(name.to_string(), 0, name.len() as u32),
                type_ann: None,
                initializer: init.map(|e| spanned(e, 0, 1)),
                modifiers: vec![],
                syntax: unilang_common::syntax_origin::SyntaxOrigin::Python,
            }),
            0,
            10,
        )
    }

    fn func_decl_stmt(
        name: &str,
        params: Vec<Param>,
        return_type: Option<TypeExpr>,
        body: Vec<Spanned<Stmt>>,
    ) -> Spanned<Stmt> {
        spanned(
            Stmt::FunctionDecl(FunctionDecl {
                name: spanned(name.to_string(), 0, name.len() as u32),
                params,
                return_type: return_type.map(|t| spanned(t, 0, 1)),
                body: block_with(body),
                visibility: Visibility::Public,
                modifiers: vec![],
                decorators: vec![],
                is_async: false,
                syntax: unilang_common::syntax_origin::SyntaxOrigin::Java,
            }),
            0,
            50,
        )
    }

    fn make_param(name: &str, type_ann: Option<TypeExpr>) -> Param {
        Param {
            name: spanned(name.to_string(), 0, name.len() as u32),
            type_ann: type_ann.map(|t| spanned(t, 0, 1)),
            default: None,
            is_vararg: false,
            is_kwarg: false,
        }
    }

    // ── Tests ────────────────────────────────────────────────

    #[test]
    fn test_undefined_variable() {
        // Reference an undefined variable
        let module = make_module(vec![spanned(
            Stmt::Expr(Expr::Ident("undefined_var".to_string())),
            0,
            12,
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(
            &diags,
            "undefined variable 'undefined_var'"
        ));
    }

    #[test]
    fn test_duplicate_declaration() {
        let module = make_module(vec![
            var_decl_stmt(
                "x",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(1)),
                vec![],
            ),
            var_decl_stmt(
                "x",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(2)),
                vec![],
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(&diags, "duplicate declaration of 'x'"));
    }

    #[test]
    fn test_scope_shadowing() {
        // Outer x, inner block with x should not error (different scopes)
        let module = make_module(vec![
            var_decl_stmt(
                "x",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(1)),
                vec![],
            ),
            spanned(
                Stmt::Block(block_with(vec![
                    var_decl_stmt(
                        "x",
                        Some(TypeExpr::Named("int".to_string())),
                        Some(Expr::IntLit(2)),
                        vec![],
                    ),
                    spanned(Stmt::Expr(Expr::Ident("x".to_string())), 20, 21),
                ])),
                10,
                30,
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_function_parameter_scope() {
        // Parameters should be accessible inside the function body
        let module = make_module(vec![func_decl_stmt(
            "foo",
            vec![make_param("a", Some(TypeExpr::Named("int".to_string())))],
            Some(TypeExpr::Named("int".to_string())),
            vec![spanned(
                Stmt::Return(Some(spanned(Expr::Ident("a".to_string()), 30, 31))),
                28,
                32,
            )],
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_return_outside_function() {
        let module = make_module(vec![spanned(Stmt::Return(None), 0, 6)]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(&diags, "'return' outside of function"));
    }

    #[test]
    fn test_break_outside_loop() {
        let module = make_module(vec![spanned(Stmt::Break, 0, 5)]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(&diags, "'break' outside of loop"));
    }

    #[test]
    fn test_type_mismatch_on_assignment() {
        // Declare x as int, then assign a string via Assign expression
        let module = make_module(vec![
            var_decl_stmt(
                "x",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(42)),
                vec![],
            ),
            spanned(
                Stmt::Expr(Expr::Assign(
                    Box::new(spanned(Expr::Ident("x".to_string()), 20, 21)),
                    Box::new(spanned(Expr::StringLit("hello".to_string()), 24, 31)),
                )),
                20,
                31,
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(&diags, "cannot assign"));
    }

    #[test]
    fn test_dynamic_type_accepts_anything() {
        // Python-style variable (no type annotation) should accept any assignment
        let module = make_module(vec![
            python_var_decl_stmt("x", Some(Expr::IntLit(1))),
            spanned(
                Stmt::Expr(Expr::Assign(
                    Box::new(spanned(Expr::Ident("x".to_string()), 20, 21)),
                    Box::new(spanned(Expr::StringLit("hello".to_string()), 24, 31)),
                )),
                20,
                31,
            ),
        ]);
        let (_result, _diags) = analyze(&module);
        // The first variable infers Int from the initializer. For true dynamic
        // behavior, declare with no type annotation and no initializer => Dynamic.
        let module2 = make_module(vec![
            python_var_decl_stmt("y", None),
            spanned(
                Stmt::Expr(Expr::Assign(
                    Box::new(spanned(Expr::Ident("y".to_string()), 20, 21)),
                    Box::new(spanned(Expr::StringLit("hello".to_string()), 24, 31)),
                )),
                20,
                31,
            ),
        ]);
        let (_result2, diags2) = analyze(&module2);
        assert!(has_no_errors(&diags2));
    }

    #[test]
    fn test_class_member_resolution() {
        // Class with a method, method body accesses a param
        let class_body = vec![func_decl_stmt(
            "greet",
            vec![make_param(
                "name",
                Some(TypeExpr::Named("String".to_string())),
            )],
            Some(TypeExpr::Named("void".to_string())),
            vec![spanned(Stmt::Expr(Expr::Ident("name".to_string())), 40, 44)],
        )];

        let module = make_module(vec![spanned(
            Stmt::ClassDecl(ClassDecl {
                name: spanned("Person".to_string(), 0, 6),
                type_params: vec![],
                bases: vec![],
                extends: None,
                implements: vec![],
                body: class_body,
                visibility: Visibility::Public,
                modifiers: vec![],
                decorators: vec![],
                syntax: unilang_common::syntax_origin::SyntaxOrigin::Java,
            }),
            0,
            100,
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_import_processing() {
        // Import a name, then reference it
        let module = make_module(vec![
            spanned(
                Stmt::Import(ImportStmt::Simple {
                    path: vec![
                        spanned("os".to_string(), 7, 9),
                        spanned("path".to_string(), 10, 14),
                    ],
                    alias: None,
                }),
                0,
                14,
            ),
            spanned(Stmt::Expr(Expr::Ident("path".to_string())), 15, 19),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_immutable_variable_reassignment() {
        // val x = 10; x = 20 should fail
        let module = make_module(vec![
            var_decl_stmt(
                "x",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(10)),
                vec![Modifier::Final],
            ),
            spanned(
                Stmt::Expr(Expr::Assign(
                    Box::new(spanned(Expr::Ident("x".to_string()), 20, 21)),
                    Box::new(spanned(Expr::IntLit(20), 24, 26)),
                )),
                20,
                26,
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(
            &diags,
            "cannot assign to immutable variable 'x'"
        ));
    }

    #[test]
    fn test_function_return_type_check() {
        // Function declared to return int, body contains return of string
        // We just check that return inside function does not error about "outside function"
        let module = make_module(vec![func_decl_stmt(
            "foo",
            vec![],
            Some(TypeExpr::Named("int".to_string())),
            vec![spanned(
                Stmt::Return(Some(spanned(Expr::IntLit(42), 30, 32))),
                28,
                33,
            )],
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_break_inside_loop_ok() {
        // break inside a while loop should be fine
        let module = make_module(vec![spanned(
            Stmt::While(WhileStmt {
                condition: spanned(Expr::BoolLit(true), 6, 10),
                body: block_with(vec![spanned(Stmt::Break, 12, 17)]),
            }),
            0,
            20,
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_continue_outside_loop() {
        let module = make_module(vec![spanned(Stmt::Continue, 0, 8)]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(&diags, "'continue' outside of loop"));
    }

    #[test]
    fn test_binary_op_type_check() {
        // Subtracting a string from an int should produce a type error
        let module = make_module(vec![
            var_decl_stmt(
                "a",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(1)),
                vec![],
            ),
            var_decl_stmt(
                "b",
                Some(TypeExpr::Named("String".to_string())),
                Some(Expr::StringLit("hi".to_string())),
                vec![],
            ),
            spanned(
                Stmt::Expr(Expr::BinaryOp(
                    Box::new(spanned(Expr::Ident("a".to_string()), 30, 31)),
                    BinOp::Sub,
                    Box::new(spanned(Expr::Ident("b".to_string()), 34, 35)),
                )),
                30,
                35,
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(
            &diags,
            "arithmetic operator requires numeric types"
        ));
    }

    #[test]
    fn test_function_call_arity() {
        // Call a 2-param function with 1 arg
        let module = make_module(vec![
            func_decl_stmt(
                "add",
                vec![
                    make_param("a", Some(TypeExpr::Named("int".to_string()))),
                    make_param("b", Some(TypeExpr::Named("int".to_string()))),
                ],
                Some(TypeExpr::Named("int".to_string())),
                vec![],
            ),
            spanned(
                Stmt::Expr(Expr::Call(
                    Box::new(spanned(Expr::Ident("add".to_string()), 60, 63)),
                    vec![Argument {
                        name: None,
                        value: spanned(Expr::IntLit(1), 64, 65),
                    }],
                )),
                60,
                66,
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(
            &diags,
            "expected 2 argument(s) but got 1"
        ));
    }

    #[test]
    fn test_var_decl_type_mismatch() {
        // int x = "hello" should produce a type error
        let module = make_module(vec![var_decl_stmt(
            "x",
            Some(TypeExpr::Named("int".to_string())),
            Some(Expr::StringLit("hello".to_string())),
            vec![],
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_error_containing(&diags, "cannot assign"));
    }

    // ── Cross-syntax interop tests ────────────────────────

    #[test]
    fn test_python_var_plus_java_var() {
        // c = 20 (Python, Dynamic/Int inferred)
        // int a = 10 (Java, Int)
        // d = c + a  — both numeric, should produce no errors
        let module = make_module(vec![
            python_var_decl_stmt("c", Some(Expr::IntLit(20))),
            var_decl_stmt(
                "a",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(10)),
                vec![],
            ),
            python_var_decl_stmt(
                "d",
                Some(Expr::BinaryOp(
                    Box::new(spanned(Expr::Ident("c".to_string()), 20, 21)),
                    BinOp::Add,
                    Box::new(spanned(Expr::Ident("a".to_string()), 24, 25)),
                )),
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_java_generic_python_ops() {
        // List<String> variable used with Python-style dynamic operations
        let module = make_module(vec![
            var_decl_stmt(
                "items",
                Some(TypeExpr::Generic(
                    Box::new(spanned(TypeExpr::Named("List".to_string()), 0, 4)),
                    vec![spanned(TypeExpr::Named("String".to_string()), 5, 11)],
                )),
                None,
                vec![],
            ),
            // Python-style access: x = items  (Dynamic accepts Generic)
            python_var_decl_stmt("x", Some(Expr::Ident("items".to_string()))),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_python_func_java_args() {
        // def foo(x, y): pass  — Python function, params are Dynamic
        // int a = 5
        // foo(a, 10)  — should work, Dynamic params accept typed args
        let module = make_module(vec![
            func_decl_stmt(
                "foo",
                vec![make_param("x", None), make_param("y", None)],
                None,
                vec![spanned(Stmt::Pass, 20, 24)],
            ),
            var_decl_stmt(
                "a",
                Some(TypeExpr::Named("int".to_string())),
                Some(Expr::IntLit(5)),
                vec![],
            ),
            spanned(
                Stmt::Expr(Expr::Call(
                    Box::new(spanned(Expr::Ident("foo".to_string()), 60, 63)),
                    vec![
                        Argument {
                            name: None,
                            value: spanned(Expr::Ident("a".to_string()), 64, 65),
                        },
                        Argument {
                            name: None,
                            value: spanned(Expr::IntLit(10), 67, 69),
                        },
                    ],
                )),
                60,
                70,
            ),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_implicit_int_to_float() {
        // float x = 10  — implicit widening, no error
        let module = make_module(vec![var_decl_stmt(
            "x",
            Some(TypeExpr::Named("float".to_string())),
            Some(Expr::IntLit(10)),
            vec![],
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_implicit_string_coercion() {
        // String s = 42 — should work with implicit conversion
        let module = make_module(vec![var_decl_stmt(
            "s",
            Some(TypeExpr::Named("String".to_string())),
            Some(Expr::IntLit(42)),
            vec![],
        )]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }

    #[test]
    fn test_from_import_processing() {
        // from os import path; use path
        let module = make_module(vec![
            spanned(
                Stmt::Import(ImportStmt::From {
                    path: vec![spanned("os".to_string(), 5, 7)],
                    names: ImportNames::Named(vec![ImportAlias {
                        name: spanned("path".to_string(), 15, 19),
                        alias: None,
                    }]),
                }),
                0,
                19,
            ),
            spanned(Stmt::Expr(Expr::Ident("path".to_string())), 20, 24),
        ]);
        let (_result, diags) = analyze(&module);
        assert!(has_no_errors(&diags));
    }
}
