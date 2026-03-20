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
            | Opcode::JumpIfTrue(ref mut t) => {
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
                // Simplified: compile the expression and pop it.
                self.compile_expr(&expr.node);
                self.emit(Opcode::Pop);
            }
            Stmt::Assert(expr, _msg) => {
                // Simplified: evaluate the expression and discard.
                self.compile_expr(&expr.node);
                self.emit(Opcode::Pop);
            }
            Stmt::Import(_) => { /* imports are a no-op at bytecode level */ }
            Stmt::Try(_) | Stmt::With(_) => {
                // Not yet supported in bytecode; silently skip.
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

        // Store as a local (or global at module level).
        let slot = self.define_local(&decl.name.node);
        self.emit(Opcode::StoreLocal(slot));
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
        // Simplified: compile the iterable, store to a local, and just
        // compile the body once. A real implementation would set up
        // an iterator protocol. For now, emit the iterable and pop it.
        self.compile_expr(&iter_expr.node);
        self.emit(Opcode::Pop);

        let loop_start = if self.in_function {
            self.fn_code.len()
        } else {
            self.code.len()
        };
        self.loop_starts.push(loop_start);
        self.loop_exits.push(Vec::new());

        // Store loop variable.
        if let Expr::Ident(name) = &target.node {
            let slot = self.define_local(name);
            self.emit(Opcode::LoadConst(Value::Null));
            self.emit(Opcode::StoreLocal(slot));
        }

        self.compile_block(body);

        self.emit(Opcode::Jump(loop_start));

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
                self.compile_expr(&lhs.node);
                self.compile_expr(&rhs.node);
                let opcode = match op {
                    BinOp::Add => Opcode::Add,
                    BinOp::Sub => Opcode::Sub,
                    BinOp::Mul => Opcode::Mul,
                    BinOp::Div => Opcode::Div,
                    BinOp::FloorDiv => Opcode::FloorDiv,
                    BinOp::Mod => Opcode::Mod,
                    BinOp::Pow => Opcode::Pow,
                    BinOp::Eq => Opcode::Eq,
                    BinOp::NotEq => Opcode::NotEq,
                    BinOp::Lt => Opcode::Lt,
                    BinOp::Gt => Opcode::Gt,
                    BinOp::LtEq => Opcode::LtEq,
                    BinOp::GtEq => Opcode::GtEq,
                    BinOp::And => Opcode::And,
                    BinOp::Or => Opcode::Or,
                    BinOp::BitAnd => Opcode::BitAnd,
                    BinOp::BitOr => Opcode::BitOr,
                    BinOp::BitXor => Opcode::BitXor,
                    BinOp::LShift => Opcode::LShift,
                    BinOp::RShift => Opcode::RShift,
                    // Unsupported ops map to a no-op (Pop + Null) for now.
                    _ => {
                        self.emit(Opcode::Pop);
                        Opcode::Pop
                    }
                };
                self.emit(opcode);
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
                self.compile_expr(&value.node);
                self.emit(Opcode::Dup);
                self.compile_assign_target(&target.node);
            }

            // Cast — just compile the expression (ignore the type at bytecode level).
            Expr::Cast(_, expr) => {
                self.compile_expr(&expr.node);
            }

            // Await — compile inner expression.
            Expr::Await(expr) => {
                self.compile_expr(&expr.node);
            }

            // List comprehension — simplified stub.
            Expr::ListComp { element, .. } => {
                self.compile_expr(&element.node);
                self.emit(Opcode::MakeList(1));
            }

            Expr::Error => {}
        }
    }

    /// Compile a function/method call.
    fn compile_call(&mut self, callee: &Expr, args: &[Argument]) {
        // Check if it is a direct `print(...)` call.
        let is_print = matches!(callee, Expr::Ident(name) if name == "print");

        if is_print {
            // Built-in print: compile each argument and emit Print.
            for arg in args {
                self.compile_expr(&arg.value.node);
                self.emit(Opcode::Print);
            }
            // print() should still leave a value on the stack for expression-statement pop.
            self.emit(Opcode::LoadConst(Value::Null));
        } else {
            // General call: push arguments, then callee, then Call.
            for arg in args {
                self.compile_expr(&arg.value.node);
            }
            self.compile_expr(callee);
            self.emit(Opcode::Call(args.len()));
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
