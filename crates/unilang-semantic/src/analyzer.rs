// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Main semantic analysis pass.
//!
//! Walks the AST to perform name resolution, scope management,
//! type checking, and declaration validation.

use unilang_common::error::{Diagnostic, DiagnosticBag};
use unilang_common::span::{SourceId, Span, Spanned};
use unilang_common::syntax_origin::SyntaxOrigin;
use unilang_parser::ast::*;

use crate::checker;
use crate::scope::{ScopeKind, ScopeStack, Symbol, SymbolKind};
use crate::types::Type;

/// The result of semantic analysis.
#[derive(Debug)]
pub struct AnalysisResult {
    /// Number of symbols defined across all scopes.
    pub symbols_defined: usize,
    /// Number of scopes created during analysis.
    pub scopes_created: usize,
}

/// Semantic analyzer that walks the AST.
pub struct Analyzer {
    scopes: ScopeStack,
    diagnostics: DiagnosticBag,
    source_id: SourceId,
    symbols_defined: usize,
    scopes_created: usize,
}

impl Analyzer {
    pub fn new(source_id: SourceId) -> Self {
        Self {
            scopes: ScopeStack::new(),
            diagnostics: DiagnosticBag::new(),
            source_id,
            symbols_defined: 0,
            scopes_created: 1, // module scope
        }
    }

    /// Run analysis on the given module and return results + diagnostics.
    pub fn analyze(mut self, module: &Module) -> (AnalysisResult, DiagnosticBag) {
        self.inject_prelude();
        for stmt in &module.statements {
            self.visit_stmt(stmt);
        }
        let result = AnalysisResult {
            symbols_defined: self.symbols_defined,
            scopes_created: self.scopes_created,
        };
        (result, self.diagnostics)
    }

    // ── Scope helpers ────────────────────────────────────────

    fn push_scope(&mut self, kind: ScopeKind) {
        self.scopes.push_scope(kind);
        self.scopes_created += 1;
    }

    fn pop_scope(&mut self) {
        self.scopes.pop_scope();
    }

    /// Names available in every module without an explicit import.
    ///
    /// Must stay in sync with `unilang_stdlib::register_all` / VM builtins. Using `Dynamic`
    /// avoids false arity errors for variadic builtins like `print`.
    fn inject_prelude(&mut self) {
        let prelude_span = Span::new(0, 0);
        const PRELUDE_FUNCS: &[&str] = &[
            // I/O
            "print",
            "println",
            "input",
            // Type conversion
            "int",
            "float",
            "str",
            "bool",
            // Type utilities
            "type",
            "isinstance",
            "hash",
            "id",
            // Collections
            "len",
            "range",
            "sorted",
            "reversed",
            "enumerate",
            "zip",
            "list",
            "dict",
            // Aggregates
            "sum",
            "any",
            "all",
            "min",
            "max",
            // Math
            "abs",
            "pow",
            "sqrt",
            "floor",
            "ceil",
            "round",
            // String utilities
            "upper",
            "lower",
            "split",
            "join",
            "strip",
            "replace",
            "contains",
            "startswith",
            "endswith",
            "format",
            // Character
            "chr",
            "ord",
            // JSON
            "json_encode",
            "json_decode",
            // File I/O
            "read_file",
            "file_exists",
            // Database (MySQL)
            "db_connect",
            "db_query",
            "db_exec",
            // Redis cache
            "redis_connect",
            "redis_get",
            "redis_set",
            "redis_setex",
            "redis_del",
            "redis_exists",
            "redis_incr",
            "redis_hset",
            "redis_hget",
            "redis_hgetall",
            "redis_hdel",
            "redis_expire",
            // Kafka events
            "kafka_produce",
            "kafka_events",
            // HTTP server
            "serve",
        ];
        for name in PRELUDE_FUNCS {
            let symbol = Symbol {
                name: (*name).to_string(),
                ty: Type::Dynamic,
                kind: SymbolKind::Function,
                mutable: false,
                span: prelude_span,
            };
            self.define_symbol(name, symbol, prelude_span);
        }
        // Java-style `System.out.println(...)` is implemented as a VM facade, not real JVM.
        for name in &["System", "None", "True", "False"] {
            let sym = Symbol {
                name: (*name).to_string(),
                ty: Type::Dynamic,
                kind: SymbolKind::Variable,
                mutable: false,
                span: prelude_span,
            };
            self.define_symbol(name, sym, prelude_span);
        }
    }

    fn define_symbol(&mut self, name: &str, symbol: Symbol, name_span: Span) {
        if let Err(existing_span) = self.scopes.define(name, symbol) {
            self.diagnostics.report(
                Diagnostic::error(format!("duplicate declaration of '{}'", name))
                    .with_code("E0200")
                    .with_label(name_span, self.source_id, "redefined here")
                    .with_label(existing_span, self.source_id, "first defined here"),
            );
        } else {
            self.symbols_defined += 1;
        }
    }

    // ── Statement visitors ───────────────────────────────────

    fn visit_stmt(&mut self, stmt: &Spanned<Stmt>) {
        match &stmt.node {
            Stmt::VarDecl(decl) => self.visit_var_decl(decl),
            Stmt::FunctionDecl(decl) => self.visit_function_decl(decl),
            Stmt::ClassDecl(decl) => self.visit_class_decl(decl),
            Stmt::Import(import) => self.visit_import(import),
            Stmt::If(if_stmt) => self.visit_if(if_stmt),
            Stmt::For(for_stmt) => self.visit_for(for_stmt),
            Stmt::While(while_stmt) => self.visit_while(while_stmt),
            Stmt::DoWhile(dw) => self.visit_do_while(dw),
            Stmt::Try(try_stmt) => self.visit_try(try_stmt),
            Stmt::With(with_stmt) => self.visit_with(with_stmt),
            Stmt::Return(expr) => self.visit_return(expr, stmt.span),
            Stmt::Throw(expr) => {
                self.visit_expr(expr);
            }
            Stmt::Break => self.visit_break(stmt.span),
            Stmt::Continue => self.visit_continue(stmt.span),
            Stmt::Pass => {}
            Stmt::Assert(expr, msg) => {
                self.visit_expr(expr);
                if let Some(m) = msg {
                    self.visit_expr(m);
                }
            }
            Stmt::Block(block) => self.visit_block(block, ScopeKind::Block),
            Stmt::Expr(expr_node) => {
                let spanned_expr = Spanned::new(expr_node.clone(), stmt.span);
                self.visit_expr(&spanned_expr);
            }
            Stmt::Error => {}
        }
    }

    fn visit_var_decl(&mut self, decl: &VarDecl) {
        // Determine the type
        let declared_type = decl
            .type_ann
            .as_ref()
            .map(|t| Type::from_type_expr(&t.node));

        // Visit the initializer to resolve names and infer type
        let init_type = decl.initializer.as_ref().map(|init| self.visit_expr(init));

        let ty = match (&declared_type, &init_type) {
            (Some(dt), Some(it)) => {
                // Check type compatibility
                if !dt.is_assignable_from(it) {
                    self.diagnostics.report(
                        Diagnostic::error(format!(
                            "cannot assign '{}' to variable of type '{}'",
                            it.display_name(),
                            dt.display_name()
                        ))
                        .with_code("E0302")
                        .with_label(
                            decl.name.span,
                            self.source_id,
                            "type mismatch",
                        ),
                    );
                }
                dt.clone()
            }
            (Some(dt), None) => dt.clone(),
            (None, Some(it)) => it.clone(),
            (None, None) => Type::Dynamic, // Python-style: no annotation, no init
        };

        let mutable = is_mutable(&decl.modifiers, &decl.syntax);

        let symbol = Symbol {
            name: decl.name.node.clone(),
            ty,
            kind: SymbolKind::Variable,
            mutable,
            span: decl.name.span,
        };
        self.define_symbol(&decl.name.node, symbol, decl.name.span);
    }

    fn visit_function_decl(&mut self, decl: &FunctionDecl) {
        // Compute function type
        let param_types: Vec<Type> = decl
            .params
            .iter()
            .map(|p| {
                p.type_ann
                    .as_ref()
                    .map(|t| Type::from_type_expr(&t.node))
                    .unwrap_or(Type::Dynamic)
            })
            .collect();
        let return_type = decl
            .return_type
            .as_ref()
            .map(|t| Type::from_type_expr(&t.node))
            .unwrap_or(Type::Dynamic);

        let fn_type = Type::Function(param_types, Box::new(return_type));

        // Register the function in the current scope
        let symbol = Symbol {
            name: decl.name.node.clone(),
            ty: fn_type,
            kind: SymbolKind::Function,
            mutable: false,
            span: decl.name.span,
        };
        self.define_symbol(&decl.name.node, symbol, decl.name.span);

        // Enter function scope, define params, visit body
        self.push_scope(ScopeKind::Function);
        for param in &decl.params {
            let ty = param
                .type_ann
                .as_ref()
                .map(|t| Type::from_type_expr(&t.node))
                .unwrap_or(Type::Dynamic);
            let sym = Symbol {
                name: param.name.node.clone(),
                ty,
                kind: SymbolKind::Parameter,
                mutable: true,
                span: param.name.span,
            };
            self.define_symbol(&param.name.node, sym, param.name.span);
        }
        for stmt in &decl.body.statements {
            self.visit_stmt(stmt);
        }
        self.pop_scope();
    }

    fn visit_class_decl(&mut self, decl: &ClassDecl) {
        // Register the class
        let symbol = Symbol {
            name: decl.name.node.clone(),
            ty: Type::Class(decl.name.node.clone()),
            kind: SymbolKind::Class,
            mutable: false,
            span: decl.name.span,
        };
        self.define_symbol(&decl.name.node, symbol, decl.name.span);

        // Enter class scope and visit body
        self.push_scope(ScopeKind::Class);
        for stmt in &decl.body {
            self.visit_stmt(stmt);
        }
        self.pop_scope();
    }

    fn visit_import(&mut self, import: &ImportStmt) {
        // Register imported names into the current scope
        match import {
            ImportStmt::Simple { path, alias } => {
                let name = alias
                    .as_ref()
                    .map(|a| a.node.clone())
                    .unwrap_or_else(|| path.last().map(|p| p.node.clone()).unwrap_or_default());
                let span = alias
                    .as_ref()
                    .map(|a| a.span)
                    .unwrap_or_else(|| path.last().map(|p| p.span).unwrap_or(Span::empty(0)));
                let symbol = Symbol {
                    name: name.clone(),
                    ty: Type::Dynamic,
                    kind: SymbolKind::Variable,
                    mutable: false,
                    span,
                };
                self.define_symbol(&name, symbol, span);
            }
            ImportStmt::From { names, .. } => match names {
                ImportNames::Named(aliases) => {
                    for alias in aliases {
                        let name = alias
                            .alias
                            .as_ref()
                            .map(|a| a.node.clone())
                            .unwrap_or_else(|| alias.name.node.clone());
                        let span = alias
                            .alias
                            .as_ref()
                            .map(|a| a.span)
                            .unwrap_or(alias.name.span);
                        let symbol = Symbol {
                            name: name.clone(),
                            ty: Type::Dynamic,
                            kind: SymbolKind::Variable,
                            mutable: false,
                            span,
                        };
                        self.define_symbol(&name, symbol, span);
                    }
                }
                ImportNames::Wildcard => {
                    // Wildcard imports are accepted but we can't resolve individual names
                }
            },
            ImportStmt::Static { path } => {
                if let Some(last) = path.last() {
                    let symbol = Symbol {
                        name: last.node.clone(),
                        ty: Type::Dynamic,
                        kind: SymbolKind::Variable,
                        mutable: false,
                        span: last.span,
                    };
                    self.define_symbol(&last.node, symbol, last.span);
                }
            }
        }
    }

    fn visit_if(&mut self, if_stmt: &IfStmt) {
        self.visit_expr(&if_stmt.condition);
        self.visit_block(&if_stmt.then_block, ScopeKind::Block);
        for (cond, block) in &if_stmt.elif_clauses {
            self.visit_expr(cond);
            self.visit_block(block, ScopeKind::Block);
        }
        if let Some(else_block) = &if_stmt.else_block {
            self.visit_block(else_block, ScopeKind::Block);
        }
    }

    fn visit_for(&mut self, for_stmt: &ForStmt) {
        match for_stmt {
            ForStmt::ForIn { target, iter, body } => {
                self.visit_expr(iter);
                self.push_scope(ScopeKind::Loop);
                // Define the loop variable
                if let Expr::Ident(name) = &target.node {
                    let symbol = Symbol {
                        name: name.clone(),
                        ty: Type::Dynamic,
                        kind: SymbolKind::Variable,
                        mutable: true,
                        span: target.span,
                    };
                    self.define_symbol(name, symbol, target.span);
                }
                for stmt in &body.statements {
                    self.visit_stmt(stmt);
                }
                self.pop_scope();
            }
            ForStmt::ForClassic {
                init,
                condition,
                update,
                body,
            } => {
                self.push_scope(ScopeKind::Loop);
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                if let Some(cond) = condition {
                    self.visit_expr(cond);
                }
                if let Some(upd) = update {
                    let spanned = Spanned::new(upd.node.clone(), upd.span);
                    self.visit_expr(&spanned);
                }
                for stmt in &body.statements {
                    self.visit_stmt(stmt);
                }
                self.pop_scope();
            }
        }
    }

    fn visit_while(&mut self, while_stmt: &WhileStmt) {
        self.visit_expr(&while_stmt.condition);
        self.push_scope(ScopeKind::Loop);
        for stmt in &while_stmt.body.statements {
            self.visit_stmt(stmt);
        }
        self.pop_scope();
    }

    fn visit_do_while(&mut self, dw: &DoWhileStmt) {
        self.push_scope(ScopeKind::Loop);
        for stmt in &dw.body.statements {
            self.visit_stmt(stmt);
        }
        self.pop_scope();
        self.visit_expr(&dw.condition);
    }

    fn visit_try(&mut self, try_stmt: &TryStmt) {
        self.visit_block(&try_stmt.body, ScopeKind::Block);
        for clause in &try_stmt.catch_clauses {
            self.push_scope(ScopeKind::Block);
            if let Some(name) = &clause.name {
                let ty = clause
                    .exception_type
                    .as_ref()
                    .map(|t| Type::from_type_expr(&t.node))
                    .unwrap_or(Type::Dynamic);
                let symbol = Symbol {
                    name: name.node.clone(),
                    ty,
                    kind: SymbolKind::Variable,
                    mutable: false,
                    span: name.span,
                };
                self.define_symbol(&name.node, symbol, name.span);
            }
            for stmt in &clause.body.statements {
                self.visit_stmt(stmt);
            }
            self.pop_scope();
        }
        if let Some(finally) = &try_stmt.finally_block {
            self.visit_block(finally, ScopeKind::Block);
        }
    }

    fn visit_with(&mut self, with_stmt: &WithStmt) {
        self.push_scope(ScopeKind::Block);
        for item in &with_stmt.items {
            self.visit_expr(&item.context);
            if let Some(alias) = &item.alias {
                let symbol = Symbol {
                    name: alias.node.clone(),
                    ty: Type::Dynamic,
                    kind: SymbolKind::Variable,
                    mutable: true,
                    span: alias.span,
                };
                self.define_symbol(&alias.node, symbol, alias.span);
            }
        }
        for stmt in &with_stmt.body.statements {
            self.visit_stmt(stmt);
        }
        self.pop_scope();
    }

    fn visit_return(&mut self, expr: &Option<Spanned<Expr>>, span: Span) {
        if !self.scopes.is_inside(ScopeKind::Function) {
            self.diagnostics.report(
                Diagnostic::error("'return' outside of function")
                    .with_code("E0201")
                    .with_label(span, self.source_id, "not inside a function"),
            );
        }
        if let Some(e) = expr {
            self.visit_expr(e);
        }
    }

    fn visit_break(&mut self, span: Span) {
        if !self.scopes.is_inside(ScopeKind::Loop) {
            self.diagnostics.report(
                Diagnostic::error("'break' outside of loop")
                    .with_code("E0202")
                    .with_label(span, self.source_id, "not inside a loop"),
            );
        }
    }

    fn visit_continue(&mut self, span: Span) {
        if !self.scopes.is_inside(ScopeKind::Loop) {
            self.diagnostics.report(
                Diagnostic::error("'continue' outside of loop")
                    .with_code("E0203")
                    .with_label(span, self.source_id, "not inside a loop"),
            );
        }
    }

    fn visit_block(&mut self, block: &Block, kind: ScopeKind) {
        self.push_scope(kind);
        for stmt in &block.statements {
            self.visit_stmt(stmt);
        }
        self.pop_scope();
    }

    // ── Expression visitors ──────────────────────────────────

    fn visit_expr(&mut self, expr: &Spanned<Expr>) -> Type {
        match &expr.node {
            Expr::IntLit(_) => Type::Int,
            Expr::FloatLit(_) => Type::Float,
            Expr::StringLit(_) => Type::String,
            Expr::BoolLit(_) => Type::Bool,
            Expr::NullLit => Type::Null,

            Expr::Ident(name) => {
                if let Some(sym) = self.scopes.resolve(name) {
                    sym.ty.clone()
                } else {
                    self.diagnostics.report(
                        Diagnostic::error(format!("undefined variable '{}'", name))
                            .with_code("E0204")
                            .with_label(expr.span, self.source_id, "not found in this scope"),
                    );
                    Type::Error
                }
            }

            Expr::Attribute(obj, _attr) => {
                self.visit_expr(obj);
                // Attribute resolution on dynamic/unknown objects returns Dynamic
                Type::Dynamic
            }

            Expr::Index(obj, index) => {
                let obj_ty = self.visit_expr(obj);
                self.visit_expr(index);
                match obj_ty {
                    Type::Array(inner) => *inner,
                    _ => Type::Dynamic,
                }
            }

            Expr::BinaryOp(left, op, right) => {
                let left_ty = self.visit_expr(left);
                let right_ty = self.visit_expr(right);
                checker::check_binary_op(
                    *op,
                    &left_ty,
                    &right_ty,
                    expr.span,
                    self.source_id,
                    &mut self.diagnostics,
                )
            }

            Expr::UnaryOp(_op, operand) => self.visit_expr(operand),

            Expr::Call(callee, args) => {
                let callee_ty = self.visit_expr(callee);
                let arg_types: Vec<Type> = args.iter().map(|a| self.visit_expr(&a.value)).collect();
                match &callee_ty {
                    Type::Function(params, ret) => {
                        checker::check_call_arity(
                            params.len(),
                            arg_types.len(),
                            expr.span,
                            self.source_id,
                            &mut self.diagnostics,
                        );
                        *ret.clone()
                    }
                    _ => Type::Dynamic,
                }
            }

            Expr::New(type_expr, args) => {
                for arg in args {
                    self.visit_expr(&arg.value);
                }
                Type::from_type_expr(&type_expr.node)
            }

            Expr::Lambda(params, body) => {
                self.push_scope(ScopeKind::Function);
                for param in params {
                    let ty = param
                        .type_ann
                        .as_ref()
                        .map(|t| Type::from_type_expr(&t.node))
                        .unwrap_or(Type::Dynamic);
                    let sym = Symbol {
                        name: param.name.node.clone(),
                        ty,
                        kind: SymbolKind::Parameter,
                        mutable: true,
                        span: param.name.span,
                    };
                    self.define_symbol(&param.name.node, sym, param.name.span);
                }
                let ret = self.visit_expr(body);
                self.pop_scope();
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        p.type_ann
                            .as_ref()
                            .map(|t| Type::from_type_expr(&t.node))
                            .unwrap_or(Type::Dynamic)
                    })
                    .collect();
                Type::Function(param_types, Box::new(ret))
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.visit_expr(condition);
                let then_ty = self.visit_expr(then_expr);
                let else_ty = self.visit_expr(else_expr);
                Type::common_type(&then_ty, &else_ty)
            }

            Expr::List(elements) => {
                let mut elem_ty = Type::Unknown;
                for elem in elements {
                    elem_ty = self.visit_expr(elem);
                }
                Type::Array(Box::new(elem_ty))
            }

            Expr::Dict(pairs) => {
                for (k, v) in pairs {
                    self.visit_expr(k);
                    self.visit_expr(v);
                }
                Type::Dynamic
            }

            Expr::Set(elements) => {
                for elem in elements {
                    self.visit_expr(elem);
                }
                Type::Dynamic
            }

            Expr::ListComp { element, clauses } => {
                self.push_scope(ScopeKind::Block);
                for clause in clauses {
                    self.visit_expr(&clause.iter);
                    if let Expr::Ident(name) = &clause.target.node {
                        let sym = Symbol {
                            name: name.clone(),
                            ty: Type::Dynamic,
                            kind: SymbolKind::Variable,
                            mutable: true,
                            span: clause.target.span,
                        };
                        self.define_symbol(name, sym, clause.target.span);
                    }
                    for cond in &clause.conditions {
                        self.visit_expr(cond);
                    }
                }
                let elem_ty = self.visit_expr(element);
                self.pop_scope();
                Type::Array(Box::new(elem_ty))
            }

            Expr::Assign(target, value) => {
                let val_ty = self.visit_expr(value);
                if let Expr::Ident(name) = &target.node {
                    if let Some(sym) = self.scopes.resolve(name) {
                        if !sym.mutable {
                            self.diagnostics.report(
                                Diagnostic::error(format!(
                                    "cannot assign to immutable variable '{}'",
                                    name
                                ))
                                .with_code("E0205")
                                .with_label(
                                    target.span,
                                    self.source_id,
                                    "variable is not mutable",
                                ),
                            );
                        }
                        checker::check_assignment_type(
                            &sym.ty,
                            &val_ty,
                            target.span,
                            self.source_id,
                            &mut self.diagnostics,
                        );
                    } else {
                        // Python-style implicit declaration: `x = expr` creates a
                        // new mutable variable when `x` is not yet in scope.
                        let symbol = Symbol {
                            name: name.clone(),
                            ty: val_ty.clone(),
                            kind: SymbolKind::Variable,
                            mutable: true,
                            span: target.span,
                        };
                        self.define_symbol(name, symbol, target.span);
                    }
                } else {
                    self.visit_expr(target);
                }
                val_ty
            }

            Expr::Cast(type_expr, inner) => {
                self.visit_expr(inner);
                Type::from_type_expr(&type_expr.node)
            }

            Expr::Await(inner) => self.visit_expr(inner),

            Expr::Error => Type::Error,
        }
    }
}

/// Determine if a variable is mutable based on its modifiers and syntax origin.
fn is_mutable(modifiers: &[Modifier], _syntax: &SyntaxOrigin) -> bool {
    // val and final are immutable; everything else is mutable by default.
    // In Java syntax, variables without `final` are mutable.
    // In Python syntax, variables are mutable by default.
    // In UniLang, `val` maps to the Final modifier.
    !modifiers.iter().any(|m| matches!(m, Modifier::Final))
}
