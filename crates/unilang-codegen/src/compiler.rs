// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! AST-to-bytecode compiler.
//!
//! Walks the UniLang AST and emits a flat stream of stack-based
//! [`Opcode`]s that a simple interpreter can execute.

use std::collections::HashMap;

use unilang_common::error::Diagnostic;
use unilang_common::span::Spanned;
use unilang_parser::ast::*;

use crate::bytecode::{Bytecode, ClassDef, Function, Opcode, Value};

/// Compiles a UniLang [`Module`] into [`Bytecode`].
pub struct Compiler {
    /// Module-level instruction stream.
    code: Vec<Opcode>,
    /// Compiled functions.
    functions: Vec<Function>,
    /// Compiled class definitions.
    classes: Vec<ClassDef>,
    /// Stack of local-variable scopes (name -> slot index).
    locals: Vec<HashMap<String, usize>>,
    /// Next available local slot in the current scope.
    local_slot: usize,
    /// Stack of loop-start instruction indices (for `continue`).
    loop_starts: Vec<usize>,
    /// Stack of pending `break` jump indices to patch (for each loop nesting).
    loop_exits: Vec<Vec<usize>>,
    /// Accumulated diagnostics.
    diagnostics: Vec<Diagnostic>,
    /// Whether we are currently compiling inside a function body.
    in_function: bool,
    /// When compiling a function body, instructions go here instead.
    fn_code: Vec<Opcode>,
}

impl Compiler {
    /// Create a new compiler.
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            locals: vec![HashMap::new()],
            local_slot: 0,
            loop_starts: Vec::new(),
            loop_exits: Vec::new(),
            diagnostics: Vec::new(),
            in_function: false,
            fn_code: Vec::new(),
        }
    }

    // ── helpers ──────────────────────────────────────────

    /// Emit an instruction and return its index.
    fn emit(&mut self, op: Opcode) -> usize {
        let code = if self.in_function {
            &mut self.fn_code
        } else {
            &mut self.code
        };
        let idx = code.len();
        code.push(op);
        idx
    }

    /// Emit a jump instruction with a placeholder target; return the
    /// index so we can patch it later.
    fn emit_jump(&mut self, op: Opcode) -> usize {
        self.emit(op)
    }

    /// Patch a previously emitted jump to point at the current position.
    fn patch_jump(&mut self, index: usize) {
        let target = if self.in_function {
            self.fn_code.len()
        } else {
            self.code.len()
        };
        let code = if self.in_function {
            &mut self.fn_code
        } else {
            &mut self.code
        };
        match &mut code[index] {
            Opcode::Jump(ref mut t)
            | Opcode::JumpIfFalse(ref mut t)
            | Opcode::JumpIfTrue(ref mut t)
            | Opcode::PushExceptHandler(ref mut t) => {
                *t = target;
            }
            _ => {}
        }
    }

    /// Push a new local scope.
    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    /// Pop the current local scope.
    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    /// Define a local variable and return its slot.
    fn define_local(&mut self, name: &str) -> usize {
        let slot = self.local_slot;
        self.local_slot += 1;
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), slot);
        }
        slot
    }

    /// Resolve a variable name to its local slot, if any.
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for scope in self.locals.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    // ── public entry point ──────────────────────────────

    /// Compile a module and return bytecode or diagnostics.
    pub fn compile_module(mut self, module: &Module) -> Result<Bytecode, Vec<Diagnostic>> {
        for stmt in &module.statements {
            self.compile_stmt(&stmt.node);
        }
        self.emit(Opcode::Halt);

        if self
            .diagnostics
            .iter()
            .any(|d| d.severity == unilang_common::error::Severity::Error)
        {
            Err(self.diagnostics)
        } else {
            Ok(Bytecode {
                instructions: self.code,
                functions: self.functions,
                classes: self.classes,
            })
        }
    }

    // ── statements ──────────────────────────────────────

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                // Discard the expression result.
                self.emit(Opcode::Pop);
            }
            Stmt::VarDecl(decl) => self.compile_var_decl(decl),
            Stmt::FunctionDecl(decl) => self.compile_function_decl(decl),
            Stmt::ClassDecl(decl) => self.compile_class_decl(decl),
            Stmt::If(if_stmt) => self.compile_if(if_stmt),
            Stmt::While(w) => self.compile_while(w),
            Stmt::DoWhile(dw) => self.compile_do_while(dw),
            Stmt::For(f) => self.compile_for(f),
            Stmt::Return(expr) => self.compile_return(expr.as_ref()),
            Stmt::Break => self.compile_break(),
            Stmt::Continue => self.compile_continue(),
            Stmt::Pass => { /* no-op */ }
            Stmt::Block(block) => self.compile_block(block),
            Stmt::Throw(expr) => {
                self.compile_expr(&expr.node);
                self.emit(Opcode::Raise);
            }
            Stmt::Assert(expr, msg) => {
                self.compile_expr(&expr.node);
                if let Some(m) = msg {
                    self.compile_expr(&m.node);
                } else {
                    self.emit(Opcode::LoadConst(Value::String(
                        "AssertionError".to_string(),
                    )));
                }
                self.emit(Opcode::Assert);
            }
            Stmt::Import(_) => { /* imports are a no-op at bytecode level */ }
            Stmt::Try(try_stmt) => {
                // ── try body ──────────────────────────────────────────────────
                // Emit PushExceptHandler(catch_ip) with a placeholder address;
                // the real address is patched after the body + jump-over are emitted.
                let handler_idx = self.emit(Opcode::PushExceptHandler(0));

                self.compile_block(&try_stmt.body);

                // After the body succeeds, remove the handler and jump over catch.
                self.emit(Opcode::PopExceptHandler);
                let jump_over_catch = self.emit_jump(Opcode::Jump(0));

                // ── catch blocks ──────────────────────────────────────────────
                // Patch the handler to point here.
                self.patch_jump(handler_idx);

                if !try_stmt.catch_clauses.is_empty() {
                    let clause = &try_stmt.catch_clauses[0];
                    // Bind the exception variable if the clause names one.
                    if let Some(exc_name) = &clause.name {
                        self.emit(Opcode::StoreExceptVar(exc_name.node.clone()));
                    } else {
                        // Discard the exception message that was pushed onto the stack.
                        self.emit(Opcode::Pop);
                    }
                    self.compile_block(&clause.body);

                    // Additional clauses (multi-catch): compile sequentially after the first.
                    // In practice most UniLang programs only have one catch clause.
                    for clause in try_stmt.catch_clauses.iter().skip(1) {
                        if let Some(exc_name) = &clause.name {
                            self.emit(Opcode::StoreExceptVar(exc_name.node.clone()));
                        } else {
                            self.emit(Opcode::Pop);
                        }
                        self.compile_block(&clause.body);
                    }
                } else {
                    // No catch clause — just discard the exception.
                    self.emit(Opcode::Pop);
                }

                // ── after catch ───────────────────────────────────────────────
                self.patch_jump(jump_over_catch);

                // ── finally ───────────────────────────────────────────────────
                if let Some(finally) = &try_stmt.finally_block {
                    self.compile_block(finally);
                }
            }
            Stmt::With(with_stmt) => {
                // Simplified: just run the body (no __enter__/__exit__ protocol yet).
                self.compile_block(&with_stmt.body);
            }
            Stmt::Error => {}
        }
    }

    fn compile_block(&mut self, block: &Block) {
        self.push_scope();
        for stmt in &block.statements {
            self.compile_stmt(&stmt.node);
        }
        self.pop_scope();
    }

    // ── declarations ────────────────────────────────────

    fn compile_var_decl(&mut self, decl: &VarDecl) {
        if let Some(init) = &decl.initializer {
            self.compile_expr(&init.node);
        } else {
            self.emit(Opcode::LoadConst(Value::Null));
        }
        let slot = self.define_local(&decl.name.node);
        self.emit(Opcode::StoreLocal(slot));
    }

    fn compile_function_decl(&mut self, decl: &FunctionDecl) {
        // Save current compiler state.
        let prev_in_function = self.in_function;
        let prev_fn_code = std::mem::take(&mut self.fn_code);
        let prev_locals = std::mem::replace(&mut self.locals, vec![HashMap::new()]);
        let prev_slot = self.local_slot;

        self.in_function = true;
        self.local_slot = 0;

        // Define parameters as locals.
        let param_names: Vec<String> = decl
            .params
            .iter()
            .map(|p| {
                let name = p.name.node.clone();
                self.define_local(&name);
                name
            })
            .collect();

        // Compile body.
        for stmt in &decl.body.statements {
            self.compile_stmt(&stmt.node);
        }

        // Ensure there is always a return.
        let needs_return = self
            .fn_code
            .last()
            .map_or(true, |op| !matches!(op, Opcode::Return));
        if needs_return {
            self.emit(Opcode::LoadConst(Value::Null));
            self.emit(Opcode::Return);
        }

        let fn_code = std::mem::take(&mut self.fn_code);
        let local_count = self.local_slot;

        // Restore state.
        self.in_function = prev_in_function;
        self.fn_code = prev_fn_code;
        self.locals = prev_locals;
        self.local_slot = prev_slot;

        let fn_index = self.functions.len();
        self.functions.push(Function {
            name: decl.name.node.clone(),
            params: param_names,
            code: fn_code,
            local_count,
        });

        // Emit instruction to make the function available at runtime.
        self.emit(Opcode::MakeFunction(fn_index));

        // At module level, store as a global so functions can call each other by name.
        // Inside a function, store as a local (nested function).
        if !prev_in_function {
            self.emit(Opcode::StoreGlobal(decl.name.node.clone()));
        } else {
            let slot = self.define_local(&decl.name.node);
            self.emit(Opcode::StoreLocal(slot));
        }
    }

    fn compile_class_decl(&mut self, decl: &ClassDecl) {
        let mut method_indices = Vec::new();
        let mut fields = Vec::new();

        for member in &decl.body {
            match &member.node {
                Stmt::FunctionDecl(method_decl) => {
                    // Save state.
                    let prev_in_function = self.in_function;
                    let prev_fn_code = std::mem::take(&mut self.fn_code);
                    let prev_locals = std::mem::replace(&mut self.locals, vec![HashMap::new()]);
                    let prev_slot = self.local_slot;

                    self.in_function = true;
                    self.local_slot = 0;

                    let param_names: Vec<String> = method_decl
                        .params
                        .iter()
                        .map(|p| {
                            let name = p.name.node.clone();
                            self.define_local(&name);
                            name
                        })
                        .collect();

                    for stmt in &method_decl.body.statements {
                        self.compile_stmt(&stmt.node);
                    }

                    let needs_return = self
                        .fn_code
                        .last()
                        .map_or(true, |op| !matches!(op, Opcode::Return));
                    if needs_return {
                        self.emit(Opcode::LoadConst(Value::Null));
                        self.emit(Opcode::Return);
                    }

                    let fn_code = std::mem::take(&mut self.fn_code);
                    let local_count = self.local_slot;

                    self.in_function = prev_in_function;
                    self.fn_code = prev_fn_code;
                    self.locals = prev_locals;
                    self.local_slot = prev_slot;

                    let fn_index = self.functions.len();
                    self.functions.push(Function {
                        name: method_decl.name.node.clone(),
                        params: param_names,
                        code: fn_code,
                        local_count,
                    });
                    method_indices.push(fn_index);
                }
                Stmt::VarDecl(var_decl) => {
                    fields.push(var_decl.name.node.clone());
                }
                _ => {}
            }
        }

        let class_name = decl.name.node.clone();
        self.classes.push(ClassDef {
            name: class_name.clone(),
            methods: method_indices,
            fields,
        });

        let member_count = self
            .classes
            .last()
            .map_or(0, |c| c.methods.len() + c.fields.len());
        self.emit(Opcode::MakeClass(class_name.clone(), member_count));

        let slot = self.define_local(&class_name);
        self.emit(Opcode::StoreLocal(slot));
    }

    // ── control flow ────────────────────────────────────

    fn compile_if(&mut self, if_stmt: &IfStmt) {
        // Compile condition.
        self.compile_expr(&if_stmt.condition.node);
        let jump_to_else = self.emit_jump(Opcode::JumpIfFalse(0));

        // Then block.
        self.compile_block(&if_stmt.then_block);

        if if_stmt.elif_clauses.is_empty() && if_stmt.else_block.is_none() {
            // Simple if with no else.
            self.patch_jump(jump_to_else);
        } else {
            // Jump over else/elif blocks after then executes.
            let jump_to_end = self.emit_jump(Opcode::Jump(0));
            self.patch_jump(jump_to_else);

            // Elif clauses.
            let mut elif_end_jumps = vec![jump_to_end];
            for (cond, block) in &if_stmt.elif_clauses {
                self.compile_expr(&cond.node);
                let jump_next = self.emit_jump(Opcode::JumpIfFalse(0));
                self.compile_block(block);
                let jmp = self.emit_jump(Opcode::Jump(0));
                elif_end_jumps.push(jmp);
                self.patch_jump(jump_next);
            }

            // Else block.
            if let Some(else_block) = &if_stmt.else_block {
                self.compile_block(else_block);
            }

            // Patch all end jumps.
            for j in elif_end_jumps {
                self.patch_jump(j);
            }
        }
    }

    fn compile_while(&mut self, w: &WhileStmt) {
        let loop_start = if self.in_function {
            self.fn_code.len()
        } else {
            self.code.len()
        };
        self.loop_starts.push(loop_start);
        self.loop_exits.push(Vec::new());

        self.compile_expr(&w.condition.node);
        let exit_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        self.compile_block(&w.body);

        self.emit(Opcode::Jump(loop_start));
        self.patch_jump(exit_jump);

        // Patch break jumps.
        let exits = self.loop_exits.pop().unwrap_or_default();
        for e in exits {
            self.patch_jump(e);
        }
        self.loop_starts.pop();
    }

    fn compile_do_while(&mut self, dw: &DoWhileStmt) {
        let loop_start = if self.in_function {
            self.fn_code.len()
        } else {
            self.code.len()
        };
        self.loop_starts.push(loop_start);
        self.loop_exits.push(Vec::new());

        self.compile_block(&dw.body);
        self.compile_expr(&dw.condition.node);
        self.emit(Opcode::JumpIfTrue(loop_start));

        let exits = self.loop_exits.pop().unwrap_or_default();
        for e in exits {
            self.patch_jump(e);
        }
        self.loop_starts.pop();
    }

    fn compile_for(&mut self, f: &ForStmt) {
        match f {
            ForStmt::ForIn { target, iter, body } => {
                self.compile_for_in(target, iter, body);
            }
            ForStmt::ForClassic {
                init,
                condition,
                update,
                body,
            } => {
                self.compile_for_classic(init, condition, update, body);
            }
        }
    }

    fn compile_for_in(&mut self, target: &Spanned<Expr>, iter_expr: &Spanned<Expr>, body: &Block) {
        // Compile and store iterable.
        self.compile_expr(&iter_expr.node);
        let iter_slot = self.define_local("__iter__");
        self.emit(Opcode::StoreLocal(iter_slot));

        // Initialize index counter to 0.
        self.emit(Opcode::LoadConst(Value::Int(0)));
        let idx_slot = self.define_local("__idx__");
        self.emit(Opcode::StoreLocal(idx_slot));

        // Define the loop target variable.
        let target_slot = if let Expr::Ident(name) = &target.node {
            let s = self.define_local(name);
            self.emit(Opcode::LoadConst(Value::Null));
            self.emit(Opcode::StoreLocal(s));
            Some(s)
        } else {
            None
        };

        // Jump to the check (skip increment on the first iteration).
        let jump_to_check = self.emit_jump(Opcode::Jump(0));

        // --- Increment step (continue target) ---
        let increment_ip = if self.in_function {
            self.fn_code.len()
        } else {
            self.code.len()
        };
        self.loop_starts.push(increment_ip);
        self.loop_exits.push(Vec::new());

        self.emit(Opcode::LoadLocal(idx_slot));
        self.emit(Opcode::LoadConst(Value::Int(1)));
        self.emit(Opcode::Add);
        self.emit(Opcode::StoreLocal(idx_slot));

        // --- Check step (patch jump_to_check here) ---
        self.patch_jump(jump_to_check);

        // Condition: idx < len(iter)
        self.emit(Opcode::LoadLocal(idx_slot));
        self.emit(Opcode::LoadLocal(iter_slot));
        self.emit(Opcode::LoadGlobal("len".to_string()));
        self.emit(Opcode::Call(1));
        self.emit(Opcode::Lt);
        let exit_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        // Load current element: iter[idx]
        self.emit(Opcode::LoadLocal(iter_slot));
        self.emit(Opcode::LoadLocal(idx_slot));
        self.emit(Opcode::GetIndex);
        if let Some(slot) = target_slot {
            self.emit(Opcode::StoreLocal(slot));
        } else {
            self.emit(Opcode::Pop);
        }

        // Body
        self.compile_block(body);

        // Jump back to increment step.
        self.emit(Opcode::Jump(increment_ip));

        // Patch exit.
        self.patch_jump(exit_jump);
        let exits = self.loop_exits.pop().unwrap_or_default();
        for e in exits {
            self.patch_jump(e);
        }
        self.loop_starts.pop();
    }

    fn compile_for_classic(
        &mut self,
        init: &Option<Box<Spanned<Stmt>>>,
        condition: &Option<Spanned<Expr>>,
        update: &Option<Spanned<Expr>>,
        body: &Block,
    ) {
        if let Some(init_stmt) = init {
            self.compile_stmt(&init_stmt.node);
        }

        let loop_start = if self.in_function {
            self.fn_code.len()
        } else {
            self.code.len()
        };
        self.loop_starts.push(loop_start);
        self.loop_exits.push(Vec::new());

        let exit_jump = if let Some(cond) = condition {
            self.compile_expr(&cond.node);
            Some(self.emit_jump(Opcode::JumpIfFalse(0)))
        } else {
            None
        };

        self.compile_block(body);

        if let Some(upd) = update {
            self.compile_expr(&upd.node);
            self.emit(Opcode::Pop);
        }

        self.emit(Opcode::Jump(loop_start));

        if let Some(ej) = exit_jump {
            self.patch_jump(ej);
        }

        let exits = self.loop_exits.pop().unwrap_or_default();
        for e in exits {
            self.patch_jump(e);
        }
        self.loop_starts.pop();
    }

    fn compile_return(&mut self, expr: Option<&Spanned<Expr>>) {
        if let Some(e) = expr {
            self.compile_expr(&e.node);
        } else {
            self.emit(Opcode::LoadConst(Value::Null));
        }
        self.emit(Opcode::Return);
    }

    fn compile_break(&mut self) {
        let j = self.emit_jump(Opcode::Jump(0));
        if let Some(exits) = self.loop_exits.last_mut() {
            exits.push(j);
        }
    }

    fn compile_continue(&mut self) {
        if let Some(&start) = self.loop_starts.last() {
            self.emit(Opcode::Jump(start));
        }
    }

    // ── expressions ─────────────────────────────────────

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            // Literals
            Expr::IntLit(v) => {
                self.emit(Opcode::LoadConst(Value::Int(*v as i64)));
            }
            Expr::FloatLit(v) => {
                self.emit(Opcode::LoadConst(Value::Float(*v)));
            }
            Expr::StringLit(s) => {
                self.emit(Opcode::LoadConst(Value::String(s.clone())));
            }
            Expr::BoolLit(b) => {
                self.emit(Opcode::LoadConst(Value::Bool(*b)));
            }
            Expr::NullLit => {
                self.emit(Opcode::LoadConst(Value::Null));
            }

            // Identifiers
            Expr::Ident(name) => {
                if name == "print" {
                    // Special-case: push a sentinel so Call knows this is print.
                    self.emit(Opcode::LoadGlobal("print".to_string()));
                } else if let Some(slot) = self.resolve_local(name) {
                    self.emit(Opcode::LoadLocal(slot));
                } else {
                    self.emit(Opcode::LoadGlobal(name.clone()));
                }
            }

            // Attribute access
            Expr::Attribute(obj, attr) => {
                self.compile_expr(&obj.node);
                self.emit(Opcode::GetAttr(attr.node.clone()));
            }

            // Index access
            Expr::Index(obj, index) => {
                self.compile_expr(&obj.node);
                self.compile_expr(&index.node);
                self.emit(Opcode::GetIndex);
            }

            // Binary operators
            Expr::BinaryOp(lhs, op, rhs) => {
                // NullCoalesce is special: lhs ?? rhs
                if matches!(op, BinOp::NullCoalesce) {
                    self.compile_expr(&lhs.node);
                    self.emit(Opcode::Dup);
                    self.emit(Opcode::LoadConst(Value::Null));
                    self.emit(Opcode::NotEq);
                    let use_lhs = self.emit_jump(Opcode::JumpIfTrue(0));
                    self.emit(Opcode::Pop);
                    self.compile_expr(&rhs.node);
                    self.patch_jump(use_lhs);
                    return;
                }

                self.compile_expr(&lhs.node);
                self.compile_expr(&rhs.node);
                match op {
                    BinOp::Add => { self.emit(Opcode::Add); }
                    BinOp::Sub => { self.emit(Opcode::Sub); }
                    BinOp::Mul => { self.emit(Opcode::Mul); }
                    BinOp::Div => { self.emit(Opcode::Div); }
                    BinOp::FloorDiv => { self.emit(Opcode::FloorDiv); }
                    BinOp::Mod => { self.emit(Opcode::Mod); }
                    BinOp::Pow => { self.emit(Opcode::Pow); }
                    BinOp::Eq => { self.emit(Opcode::Eq); }
                    BinOp::NotEq => { self.emit(Opcode::NotEq); }
                    BinOp::Lt => { self.emit(Opcode::Lt); }
                    BinOp::Gt => { self.emit(Opcode::Gt); }
                    BinOp::LtEq => { self.emit(Opcode::LtEq); }
                    BinOp::GtEq => { self.emit(Opcode::GtEq); }
                    BinOp::And => { self.emit(Opcode::And); }
                    BinOp::Or => { self.emit(Opcode::Or); }
                    BinOp::BitAnd => { self.emit(Opcode::BitAnd); }
                    BinOp::BitOr => { self.emit(Opcode::BitOr); }
                    BinOp::BitXor => { self.emit(Opcode::BitXor); }
                    BinOp::LShift => { self.emit(Opcode::LShift); }
                    BinOp::RShift => { self.emit(Opcode::RShift); }
                    BinOp::UnsignedRShift => { self.emit(Opcode::RShift); }
                    BinOp::In => { self.emit(Opcode::Contains); }
                    BinOp::NotIn => { self.emit(Opcode::Contains); self.emit(Opcode::Not); }
                    BinOp::Is => { self.emit(Opcode::Eq); }
                    BinOp::IsNot => { self.emit(Opcode::Eq); self.emit(Opcode::Not); }
                    BinOp::Instanceof => { self.emit(Opcode::Eq); }
                    BinOp::NullCoalesce => unreachable!(),
                };
            }

            // Unary operators
            Expr::UnaryOp(op, operand) => {
                self.compile_expr(&operand.node);
                let opcode = match op {
                    UnaryOp::Neg => Opcode::Neg,
                    UnaryOp::Not | UnaryOp::LogicalNot => Opcode::Not,
                    UnaryOp::BitNot => Opcode::BitNot,
                    UnaryOp::Pos => {
                        // Unary plus is a no-op.
                        return;
                    }
                };
                self.emit(opcode);
            }

            // Function / method call
            Expr::Call(callee, args) => {
                self.compile_call(&callee.node, args);
            }

            // new ClassName(args)
            Expr::New(type_expr, args) => {
                let class_name = match &type_expr.node {
                    TypeExpr::Named(n) => n.clone(),
                    _ => "Unknown".to_string(),
                };
                for arg in args {
                    self.compile_expr(&arg.value.node);
                }
                self.emit(Opcode::NewInstance(class_name));
            }

            // Lambda
            Expr::Lambda(params, body) => {
                let prev_in_function = self.in_function;
                let prev_fn_code = std::mem::take(&mut self.fn_code);
                let prev_locals = std::mem::replace(&mut self.locals, vec![HashMap::new()]);
                let prev_slot = self.local_slot;

                self.in_function = true;
                self.local_slot = 0;

                let param_names: Vec<String> = params
                    .iter()
                    .map(|p| {
                        let name = p.name.node.clone();
                        self.define_local(&name);
                        name
                    })
                    .collect();

                self.compile_expr(&body.node);
                self.emit(Opcode::Return);

                let fn_code = std::mem::take(&mut self.fn_code);
                let local_count = self.local_slot;

                self.in_function = prev_in_function;
                self.fn_code = prev_fn_code;
                self.locals = prev_locals;
                self.local_slot = prev_slot;

                let fn_index = self.functions.len();
                self.functions.push(Function {
                    name: "<lambda>".to_string(),
                    params: param_names,
                    code: fn_code,
                    local_count,
                });
                self.emit(Opcode::MakeFunction(fn_index));
            }

            // Ternary
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expr(&condition.node);
                let jump_else = self.emit_jump(Opcode::JumpIfFalse(0));
                self.compile_expr(&then_expr.node);
                let jump_end = self.emit_jump(Opcode::Jump(0));
                self.patch_jump(jump_else);
                self.compile_expr(&else_expr.node);
                self.patch_jump(jump_end);
            }

            // Collections
            Expr::List(items) => {
                for item in items {
                    self.compile_expr(&item.node);
                }
                self.emit(Opcode::MakeList(items.len()));
            }
            Expr::Dict(pairs) => {
                for (k, v) in pairs {
                    self.compile_expr(&k.node);
                    self.compile_expr(&v.node);
                }
                self.emit(Opcode::MakeDict(pairs.len()));
            }
            Expr::Set(items) => {
                // Treat sets as lists for now.
                for item in items {
                    self.compile_expr(&item.node);
                }
                self.emit(Opcode::MakeList(items.len()));
            }

            // Assignment
            Expr::Assign(target, value) => {
                if let Expr::Index(obj, index) = &target.node {
                    // Index assignment: a[i] = v
                    // SetIndex expects stack: [collection, index, value] (bottom to top)
                    self.compile_expr(&obj.node);     // push collection
                    self.compile_expr(&index.node);   // push index
                    self.compile_expr(&value.node);   // push value
                    self.emit(Opcode::SetIndex);       // pops value, index, collection; pushes modified collection
                    // Store the modified collection back to its source variable.
                    self.store_back_to_source(&obj.node);
                    // Push Null as the expression result.
                    self.emit(Opcode::LoadConst(Value::Null));
                } else if let Expr::Attribute(obj_expr, attr) = &target.node {
                    // Attribute assignment: obj.attr = value
                    // SetAttr expects stack: [obj, value] (bottom to top): pops value first, then obj
                    self.compile_expr(&obj_expr.node);              // push obj
                    self.compile_expr(&value.node);                 // push value
                    self.emit(Opcode::SetAttr(attr.node.clone()));  // pops value, pops obj, pushes modified obj
                    // Store modified obj back to its source variable (e.g. 'this' or any named var).
                    self.store_back_to_source(&obj_expr.node);
                    // Leave the assigned value on stack as the expression result.
                    self.compile_expr(&value.node);
                } else {
                    self.compile_expr(&value.node);
                    self.emit(Opcode::Dup);
                    self.compile_assign_target(&target.node);
                }
            }

            // Cast — just compile the expression (ignore the type at bytecode level).
            Expr::Cast(_, expr) => {
                self.compile_expr(&expr.node);
            }

            // Await — compile inner expression.
            Expr::Await(expr) => {
                self.compile_expr(&expr.node);
            }

            // List comprehension: [element for target in iter if condition]
            Expr::ListComp { element, clauses } => {
                if let Some(clause) = clauses.first() {
                    // Create empty result list.
                    self.emit(Opcode::MakeList(0));
                    let result_slot = self.define_local("__comp_result__");
                    self.emit(Opcode::StoreLocal(result_slot));

                    // Compile iterable.
                    self.compile_expr(&clause.iter.node);
                    let iter_slot = self.define_local("__comp_iter__");
                    self.emit(Opcode::StoreLocal(iter_slot));

                    // Index counter.
                    self.emit(Opcode::LoadConst(Value::Int(0)));
                    let idx_slot = self.define_local("__comp_idx__");
                    self.emit(Opcode::StoreLocal(idx_slot));

                    // Target variable.
                    let target_slot = if let Expr::Ident(name) = &clause.target.node {
                        let s = self.define_local(name);
                        self.emit(Opcode::LoadConst(Value::Null));
                        self.emit(Opcode::StoreLocal(s));
                        Some(s)
                    } else {
                        None
                    };

                    // Jump to check.
                    let jump_to_check = self.emit_jump(Opcode::Jump(0));

                    // Increment.
                    let increment_ip = if self.in_function {
                        self.fn_code.len()
                    } else {
                        self.code.len()
                    };
                    self.emit(Opcode::LoadLocal(idx_slot));
                    self.emit(Opcode::LoadConst(Value::Int(1)));
                    self.emit(Opcode::Add);
                    self.emit(Opcode::StoreLocal(idx_slot));

                    // Check.
                    self.patch_jump(jump_to_check);
                    self.emit(Opcode::LoadLocal(idx_slot));
                    self.emit(Opcode::LoadLocal(iter_slot));
                    self.emit(Opcode::LoadGlobal("len".to_string()));
                    self.emit(Opcode::Call(1));
                    self.emit(Opcode::Lt);
                    let exit_jump = self.emit_jump(Opcode::JumpIfFalse(0));

                    // Load item.
                    self.emit(Opcode::LoadLocal(iter_slot));
                    self.emit(Opcode::LoadLocal(idx_slot));
                    self.emit(Opcode::GetIndex);
                    if let Some(s) = target_slot {
                        self.emit(Opcode::StoreLocal(s));
                    } else {
                        self.emit(Opcode::Pop);
                    }

                    // Optional filter conditions.
                    let mut cond_skips: Vec<usize> = Vec::new();
                    for cond in &clause.conditions {
                        self.compile_expr(&cond.node);
                        cond_skips.push(self.emit_jump(Opcode::JumpIfFalse(0)));
                    }

                    // Append element to result list.
                    self.emit(Opcode::LoadLocal(result_slot));
                    self.compile_expr(&element.node);
                    self.emit(Opcode::CallMethod("append".to_string(), 1));
                    self.emit(Opcode::StoreLocal(result_slot));

                    for s in cond_skips {
                        self.patch_jump(s);
                    }

                    self.emit(Opcode::Jump(increment_ip));
                    self.patch_jump(exit_jump);

                    // Push final result list.
                    self.emit(Opcode::LoadLocal(result_slot));
                } else {
                    self.emit(Opcode::MakeList(0));
                }
            }

            Expr::Error => {}
        }
    }

    /// True if a method name mutates the receiver (list/dict operations).
    fn is_mutating_method(name: &str) -> bool {
        matches!(
            name,
            "append"
                | "pop"
                | "insert"
                | "remove"
                | "extend"
                | "sort"
                | "reverse"
                | "clear"
                | "put"
                | "delete"
        )
    }

    /// Compile a function/method call.
    fn compile_call(&mut self, callee: &Expr, args: &[Argument]) {
        // Method call: callee is `obj.method`
        if let Expr::Attribute(obj, method_name) = callee {
            let method = method_name.node.clone();
            if Self::is_mutating_method(&method) {
                // Mutating methods: load, call, store back, return Null.
                if let Expr::Ident(var_name) = &obj.node {
                    if let Some(slot) = self.resolve_local(var_name) {
                        self.emit(Opcode::LoadLocal(slot));
                        for arg in args {
                            self.compile_expr(&arg.value.node);
                        }
                        self.emit(Opcode::CallMethod(method, args.len()));
                        // CallMethod for mutating ops returns the modified object.
                        self.emit(Opcode::StoreLocal(slot));
                    } else {
                        self.emit(Opcode::LoadGlobal(var_name.clone()));
                        for arg in args {
                            self.compile_expr(&arg.value.node);
                        }
                        self.emit(Opcode::CallMethod(method, args.len()));
                        self.emit(Opcode::StoreGlobal(var_name.clone()));
                    }
                    self.emit(Opcode::LoadConst(Value::Null));
                    return;
                }
            }
            // Non-mutating (or non-ident receiver): compile obj, args, CallMethod.
            self.compile_expr(&obj.node);
            for arg in args {
                self.compile_expr(&arg.value.node);
            }
            self.emit(Opcode::CallMethod(method, args.len()));
            return;
        }

        // Direct `print(...)` call — single arg uses Print opcode, multi-arg uses Call.
        if let Expr::Ident(name) = callee {
            if name == "print" {
                if args.len() == 1 {
                    self.compile_expr(&args[0].value.node);
                    self.emit(Opcode::Print);
                    self.emit(Opcode::LoadConst(Value::Null));
                } else {
                    // 0 or 2+ args: route through Call so they're joined with spaces.
                    for arg in args {
                        self.compile_expr(&arg.value.node);
                    }
                    self.emit(Opcode::LoadGlobal("print".to_string()));
                    self.emit(Opcode::Call(args.len()));
                }
                return;
            }
        }

        // General call: push arguments, then callee, then Call.
        for arg in args {
            self.compile_expr(&arg.value.node);
        }
        self.compile_expr(callee);
        self.emit(Opcode::Call(args.len()));
    }

    /// Emit instructions to store the top-of-stack value back into `obj`.
    /// Used after `SetIndex` to persist the modified collection.
    fn store_back_to_source(&mut self, obj: &Expr) {
        match obj {
            Expr::Ident(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(Opcode::StoreLocal(slot));
                } else {
                    self.emit(Opcode::StoreGlobal(name.clone()));
                }
            }
            Expr::Index(parent, index) => {
                // Nested index: e.g. a[i]["key"] = v.
                // After inner SetIndex, the modified inner collection is on stack.
                // We need to store it back at a[i], then recurse.
                // Use a temp slot to avoid stack ordering issues.
                let temp_slot = self.define_local("__nested_tmp__");
                self.emit(Opcode::StoreLocal(temp_slot)); // save modified inner value
                // Compile outer assignment: parent[index] = modified_value
                self.compile_expr(&parent.node);        // push parent collection
                self.compile_expr(&index.node);          // push index
                self.emit(Opcode::LoadLocal(temp_slot)); // push saved modified value
                self.emit(Opcode::SetIndex);              // pops value, index, parent; pushes modified parent
                // Recursively store back to parent's source
                self.store_back_to_source(&parent.node);
            }
            _ => {
                // Other complex lhs: pop the result to avoid stack leak
                self.emit(Opcode::Pop);
            }
        }
    }

    /// Emit store instructions for an assignment target.
    fn compile_assign_target(&mut self, target: &Expr) {
        match target {
            Expr::Ident(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(Opcode::StoreLocal(slot));
                } else {
                    self.emit(Opcode::StoreGlobal(name.clone()));
                }
            }
            Expr::Attribute(obj, attr) => {
                self.compile_expr(&obj.node);
                self.emit(Opcode::SetAttr(attr.node.clone()));
            }
            Expr::Index(obj, index) => {
                self.compile_expr(&obj.node);
                self.compile_expr(&index.node);
                self.emit(Opcode::SetIndex);
            }
            _ => {}
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
