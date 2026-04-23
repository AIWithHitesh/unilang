// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # unilang-jvm
//!
//! Compiles a UniLang AST into a valid JVM `.class` file (Java 11, class
//! version 55.0).
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use unilang_jvm::unilang_compile_to_jvm;
//!
//! let source = r#"
//! x: int = 40 + 2
//! print(x)
//! "#;
//!
//! let bytes = unilang_compile_to_jvm(source, "Hello").unwrap();
//! std::fs::write("Hello.class", bytes).unwrap();
//! // $ java Hello   =>  42
//! ```
//!
//! ## Supported subset
//! - Integer and float literals / arithmetic
//! - String literals, `print()` calls
//! - Top-level `def` functions compiled to `public static` methods
//! - Top-level variable declarations (stored in locals of `main`)
//! - `if` / `else` control flow
//! - `while` loops
//! - `return` statements

pub mod constant_pool;
pub mod emitter;
pub mod opcodes;

use constant_pool::ConstantPool;
use emitter::Emitter;
use unilang_common::span::Spanned;
use unilang_parser::ast::*;

// Bring in specific opcode constants we need.
use opcodes::{
    ACC_PUBLIC, ACC_STATIC, ACC_SUPER, ACONST_NULL, D2I, DADD, DCMPG, DCONST_0, DDIV, DMUL, DNEG,
    DREM, DSUB, GOTO, I2D, IADD, ICONST_0, ICONST_1, IDIV, IFEQ, IFGE, IFGT, IFLE, IFLT, IFNE,
    IF_ICMPEQ, IF_ICMPGE, IF_ICMPGT, IF_ICMPLE, IF_ICMPLT, IF_ICMPNE, IMUL, INEG, IREM, ISUB,
};

// Bitwise opcodes not in the module (defined here as private constants).
const IAND: u8 = 0x7e;
const IOR: u8 = 0x80;
const IXOR: u8 = 0x82;

// ── Public error type ─────────────────────────────────────────────────────────

/// Error produced during JVM compilation.
#[derive(Debug, Clone)]
pub struct JvmError(pub String);

impl std::fmt::Display for JvmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "jvm compile error: {}", self.0)
    }
}

impl std::error::Error for JvmError {}

// ── JVM type descriptors ──────────────────────────────────────────────────────

/// Category of a JVM type — controls which load/store/arithmetic opcodes to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JvmType {
    Int,    // int  — descriptor "I"
    Double, // double — descriptor "D"
    Bool,   // boolean — descriptor "Z" (stored as int on JVM)
    Ref,    // reference (Object / String) — descriptor "Ljava/lang/Object;"
    Void,   // void — descriptor "V"
}

impl JvmType {
    /// JVM field/method descriptor string for this type.
    pub fn descriptor(self) -> &'static str {
        match self {
            JvmType::Int => "I",
            JvmType::Double => "D",
            JvmType::Bool => "Z",
            JvmType::Ref => "Ljava/lang/Object;",
            JvmType::Void => "V",
        }
    }

    /// Number of JVM local-variable slots consumed.
    pub fn slots(self) -> u8 {
        match self {
            JvmType::Double => 2,
            _ => 1,
        }
    }
}

/// Derive a `JvmType` from an optional UniLang type annotation.
fn type_from_ann(ann: Option<&TypeExpr>) -> JvmType {
    match ann {
        None => JvmType::Ref,
        Some(TypeExpr::Named(s)) => match s.as_str() {
            "int" | "long" | "short" | "byte" | "char" => JvmType::Int,
            "float" | "double" => JvmType::Double,
            "bool" | "boolean" => JvmType::Bool,
            "void" => JvmType::Void,
            _ => JvmType::Ref,
        },
        _ => JvmType::Ref,
    }
}

// ── Local variable table ──────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct Locals {
    /// (name, slot_index, type)
    slots: Vec<(String, u8, JvmType)>,
    next_slot: u8,
}

impl Locals {
    fn new() -> Self {
        Self::default()
    }

    /// Allocate a new named local and return its slot index.
    fn alloc(&mut self, name: &str, ty: JvmType) -> u8 {
        let slot = self.next_slot;
        self.slots.push((name.to_string(), slot, ty));
        self.next_slot = self.next_slot.saturating_add(ty.slots());
        slot
    }

    /// Look up a local by name.  Returns `(slot_index, type)` if found.
    fn get(&self, name: &str) -> Option<(u8, JvmType)> {
        self.slots
            .iter()
            .rev()
            .find(|(n, _, _)| n == name)
            .map(|(_, slot, ty)| (*slot, *ty))
    }

    fn max_locals(&self) -> u16 {
        self.next_slot as u16
    }
}

// ── Method compilation context ────────────────────────────────────────────────

struct MethodCtx<'cp> {
    cp: &'cp mut ConstantPool,
    em: Emitter,
    locals: Locals,
    max_stack: u16,
    cur_stack: u16,
}

impl<'cp> MethodCtx<'cp> {
    fn new(cp: &'cp mut ConstantPool) -> Self {
        Self {
            cp,
            em: Emitter::new(),
            locals: Locals::new(),
            max_stack: 8,
            cur_stack: 0,
        }
    }

    fn push(&mut self, n: u16) {
        self.cur_stack += n;
        if self.cur_stack > self.max_stack {
            self.max_stack = self.cur_stack;
        }
    }

    fn pop(&mut self, n: u16) {
        self.cur_stack = self.cur_stack.saturating_sub(n);
    }
}

// ── JVM compiler ─────────────────────────────────────────────────────────────

/// JVM bytecode compiler for a UniLang `Module`.
pub struct JvmCompiler<'m> {
    module: &'m Module,
    class_name: String,
    cp: ConstantPool,
}

impl<'m> JvmCompiler<'m> {
    /// Create a new compiler for `module` that will produce a class named
    /// `class_name` (e.g. `"Hello"` or `"com/example/Hello"`).
    pub fn new(module: &'m Module, class_name: &str) -> Self {
        Self {
            module,
            class_name: class_name.replace('.', "/"),
            cp: ConstantPool::new(),
        }
    }

    /// Compile the module and return raw `.class` bytes.
    pub fn compile(module: &'m Module, class_name: &str) -> Result<Vec<u8>, JvmError> {
        let mut c = JvmCompiler::new(module, class_name);
        c.do_compile()
    }

    fn do_compile(&mut self) -> Result<Vec<u8>, JvmError> {
        // Pre-register this_class and super_class before any methods can
        // reference them (so their cp indices are stable).
        let this_class = self.cp.class(&self.class_name.clone());
        let super_class = self.cp.class("java/lang/Object");

        // Compile all top-level function declarations first.
        let mut methods: Vec<Vec<u8>> = Vec::new();
        for stmt in &self.module.statements {
            if let Stmt::FunctionDecl(fd) = &stmt.node {
                let m = self.compile_function(fd)?;
                methods.push(m);
            }
        }

        // Compile `main`.
        let main_m = self.compile_main()?;
        methods.push(main_m);

        // ── Assemble class file ───────────────────────────────────────────────
        let mut out: Vec<u8> = Vec::new();

        // Magic
        out.extend_from_slice(&0xCAFE_BABEu32.to_be_bytes());
        // Minor version = 0, major version = 55 (Java 11)
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&55u16.to_be_bytes());
        // Constant pool (count is stored as len+1 due to 1-indexing)
        let cp_count = self.cp.len() + 1;
        out.extend_from_slice(&cp_count.to_be_bytes());
        self.cp.write_to(&mut out);
        // Access flags: ACC_PUBLIC | ACC_SUPER
        out.extend_from_slice(&(ACC_PUBLIC | ACC_SUPER).to_be_bytes());
        // this_class, super_class
        out.extend_from_slice(&this_class.to_be_bytes());
        out.extend_from_slice(&super_class.to_be_bytes());
        // interfaces_count, fields_count = 0
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        // methods
        out.extend_from_slice(&(methods.len() as u16).to_be_bytes());
        for m in &methods {
            out.extend_from_slice(m);
        }
        // attributes_count = 0
        out.extend_from_slice(&0u16.to_be_bytes());

        Ok(out)
    }

    // ── main method ───────────────────────────────────────────────────────────

    fn compile_main(&mut self) -> Result<Vec<u8>, JvmError> {
        // Extract cp into a local so MethodCtx borrows it rather than self,
        // allowing the helper methods below to borrow self independently.
        let mut cp = std::mem::replace(&mut self.cp, ConstantPool::new());
        let result = (|| {
            let mut ctx = MethodCtx::new(&mut cp);
            ctx.locals.alloc("$args", JvmType::Ref);

            for stmt in &self.module.statements {
                if matches!(&stmt.node, Stmt::FunctionDecl(_)) {
                    continue;
                }
                self.compile_stmt(&mut ctx, &stmt.node)?;
            }

            ctx.em.emit_return();
            let code = ctx.em.into_bytes();
            let max_locals = ctx.locals.max_locals().max(1);
            let max_stack = ctx.max_stack.max(4);
            Ok::<(Vec<u8>, u16, u16), JvmError>((code, max_stack, max_locals))
        })();
        self.cp = cp;
        let (code, max_stack, max_locals) = result?;
        Ok(self.build_method(
            ACC_PUBLIC | ACC_STATIC,
            "main",
            "([Ljava/lang/String;)V",
            &code,
            max_stack,
            max_locals,
        ))
    }

    // ── function → static method ──────────────────────────────────────────────

    fn compile_function(&mut self, fd: &FunctionDecl) -> Result<Vec<u8>, JvmError> {
        // Same split-borrow trick as compile_main: extract cp into a local.
        let mut cp = std::mem::replace(&mut self.cp, ConstantPool::new());
        let result = (|| {
            let mut ctx = MethodCtx::new(&mut cp);

            let mut param_desc = String::from("(");
            for param in &fd.params {
                if param.name.node == "self" {
                    continue;
                }
                let jty = type_from_ann(param.type_ann.as_ref().map(|s| &s.node));
                param_desc.push_str(jty.descriptor());
                ctx.locals.alloc(&param.name.node, jty);
            }
            param_desc.push(')');

            let ret_ty = type_from_ann(fd.return_type.as_ref().map(|s| &s.node));
            param_desc.push_str(ret_ty.descriptor());

            for stmt in &fd.body.statements {
                self.compile_stmt(&mut ctx, &stmt.node)?;
            }

            // Fallthrough return.
            match ret_ty {
                JvmType::Void => ctx.em.emit_return(),
                JvmType::Int | JvmType::Bool => {
                    ctx.em.emit_u8(ICONST_0);
                    ctx.em.emit_ireturn();
                }
                JvmType::Double => {
                    ctx.em.emit_u8(DCONST_0);
                    ctx.em.emit_dreturn();
                }
                JvmType::Ref => {
                    ctx.em.emit_u8(ACONST_NULL);
                    ctx.em.emit_areturn();
                }
            }

            let code = ctx.em.into_bytes();
            let max_locals = ctx.locals.max_locals().max(1);
            let max_stack = ctx.max_stack.max(4);
            Ok::<(Vec<u8>, u16, u16, String, String), JvmError>((
                code,
                max_stack,
                max_locals,
                fd.name.node.clone(),
                param_desc,
            ))
        })();
        self.cp = cp;
        let (code, max_stack, max_locals, fn_name, param_desc) = result?;
        Ok(self.build_method(
            ACC_PUBLIC | ACC_STATIC,
            &fn_name,
            &param_desc,
            &code,
            max_stack,
            max_locals,
        ))
    }

    // ── Statement compilation ─────────────────────────────────────────────────

    fn compile_stmt(&mut self, ctx: &mut MethodCtx, stmt: &Stmt) -> Result<(), JvmError> {
        match stmt {
            Stmt::Expr(expr) => {
                let ty = self.compile_expr(ctx, expr)?;
                // Pop any value left on the stack by an expression statement.
                // Double-wide types need a two-slot pop (pop2 = 0x58), but we
                // use two POP instructions for simplicity and correctness.
                match ty {
                    JvmType::Double => {
                        ctx.em.emit_u8(opcodes::POP2);
                        ctx.pop(2);
                    }
                    JvmType::Void => {} // nothing to pop
                    _ => {
                        ctx.em.emit_pop();
                        ctx.pop(1);
                    }
                }
            }

            Stmt::VarDecl(vd) => {
                let ty = type_from_ann(vd.type_ann.as_ref().map(|s| &s.node));
                if let Some(init) = &vd.initializer {
                    let actual = self.compile_expr(ctx, &init.node)?;
                    let target_ty = if ty == JvmType::Ref && actual != JvmType::Ref {
                        // Infer from actual type when annotation is absent.
                        actual
                    } else {
                        ty
                    };
                    self.coerce_stack(ctx, actual, target_ty);
                    let slot = ctx.locals.alloc(&vd.name.node, target_ty);
                    ctx.pop(target_ty.slots() as u16);
                    self.emit_store(ctx, slot, target_ty);
                } else {
                    // No initializer — just reserve the slot.
                    ctx.locals.alloc(&vd.name.node, ty);
                }
            }

            Stmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let ty = self.compile_expr(ctx, &expr.node)?;
                    ctx.pop(ty.slots() as u16);
                    self.emit_typed_return(ctx, ty);
                } else {
                    ctx.em.emit_return();
                }
            }

            Stmt::If(if_stmt) => self.compile_if(ctx, if_stmt)?,

            Stmt::While(ws) => self.compile_while(ctx, ws)?,

            Stmt::Block(block) => {
                for s in &block.statements {
                    self.compile_stmt(ctx, &s.node)?;
                }
            }

            Stmt::FunctionDecl(_) | Stmt::ClassDecl(_) | Stmt::Import(_) => {
                // These are not supported inside method bodies yet.
            }

            Stmt::Pass => {}

            _ => {} // unsupported — skip silently
        }
        Ok(())
    }

    fn emit_typed_return(&mut self, ctx: &mut MethodCtx, ty: JvmType) {
        match ty {
            JvmType::Int | JvmType::Bool => ctx.em.emit_ireturn(),
            JvmType::Double => ctx.em.emit_dreturn(),
            JvmType::Ref => ctx.em.emit_areturn(),
            JvmType::Void => ctx.em.emit_return(),
        }
    }

    fn emit_store(&mut self, ctx: &mut MethodCtx, slot: u8, ty: JvmType) {
        match ty {
            JvmType::Int | JvmType::Bool => ctx.em.emit_istore(slot),
            JvmType::Double => ctx.em.emit_dstore(slot),
            JvmType::Ref | JvmType::Void => ctx.em.emit_astore(slot),
        }
    }

    fn emit_load(&mut self, ctx: &mut MethodCtx, slot: u8, ty: JvmType) {
        match ty {
            JvmType::Int | JvmType::Bool => ctx.em.emit_iload(slot),
            JvmType::Double => ctx.em.emit_dload(slot),
            JvmType::Ref | JvmType::Void => ctx.em.emit_aload(slot),
        }
        ctx.push(ty.slots() as u16);
    }

    // ── Expression compilation ────────────────────────────────────────────────

    /// Compile `expr` and return the JVM type left on the operand stack.
    fn compile_expr(&mut self, ctx: &mut MethodCtx, expr: &Expr) -> Result<JvmType, JvmError> {
        match expr {
            // ── Literals ──────────────────────────────────────────────────────
            Expr::IntLit(n) => {
                let v = *n as i32;
                if (-1..=5).contains(&v)
                    || (-128..=127).contains(&v)
                    || (-32768..=32767).contains(&v)
                {
                    ctx.em.emit_int_const(v, None);
                } else {
                    let idx = ctx.cp.integer(v);
                    ctx.em.emit_ldc_w(idx);
                }
                ctx.push(1);
                Ok(JvmType::Int)
            }

            Expr::FloatLit(f) => {
                match *f {
                    v if v == 0.0 => ctx.em.emit_u8(DCONST_0),
                    v if v == 1.0 => ctx.em.emit_u8(opcodes::DCONST_1),
                    _ => {
                        let idx = ctx.cp.double(*f);
                        ctx.em.emit_ldc2_w(idx);
                    }
                }
                ctx.push(2);
                Ok(JvmType::Double)
            }

            Expr::BoolLit(b) => {
                ctx.em.emit_u8(if *b { ICONST_1 } else { ICONST_0 });
                ctx.push(1);
                Ok(JvmType::Bool)
            }

            Expr::NullLit => {
                ctx.em.emit_u8(ACONST_NULL);
                ctx.push(1);
                Ok(JvmType::Ref)
            }

            Expr::StringLit(s) => {
                let idx = ctx.cp.string(s);
                ctx.em.emit_ldc(idx);
                ctx.push(1);
                Ok(JvmType::Ref)
            }

            // ── Variable load ─────────────────────────────────────────────────
            Expr::Ident(name) => {
                if let Some((slot, ty)) = ctx.locals.get(name) {
                    self.emit_load(ctx, slot, ty);
                    Ok(ty)
                } else {
                    // Unknown identifier — push null.
                    ctx.em.emit_u8(ACONST_NULL);
                    ctx.push(1);
                    Ok(JvmType::Ref)
                }
            }

            // ── Assignment expression ─────────────────────────────────────────
            Expr::Assign(lhs, rhs) => {
                if let Expr::Ident(name) = &lhs.node {
                    let rhs_ty = self.compile_expr(ctx, &rhs.node)?;
                    // dup so the expression has a value after the store.
                    if rhs_ty == JvmType::Double {
                        ctx.em.emit_u8(opcodes::DUP2);
                        ctx.push(2);
                    } else {
                        ctx.em.emit_dup();
                        ctx.push(1);
                    }
                    if let Some((slot, ty)) = ctx.locals.get(name) {
                        self.coerce_stack(ctx, rhs_ty, ty);
                        ctx.pop(ty.slots() as u16);
                        self.emit_store(ctx, slot, ty);
                        ctx.pop(rhs_ty.slots() as u16);
                        Ok(ty)
                    } else {
                        // Allocate a new local.
                        let slot = ctx.locals.alloc(name, rhs_ty);
                        ctx.pop(rhs_ty.slots() as u16);
                        self.emit_store(ctx, slot, rhs_ty);
                        Ok(rhs_ty)
                    }
                } else {
                    // Unsupported LHS — compile rhs as value.
                    self.compile_expr(ctx, &rhs.node)
                }
            }

            // ── Binary operations ─────────────────────────────────────────────
            Expr::BinaryOp(lhs, op, rhs) => self.compile_binop(ctx, lhs, *op, rhs),

            // ── Unary operations ──────────────────────────────────────────────
            Expr::UnaryOp(op, operand) => {
                let ty = self.compile_expr(ctx, &operand.node)?;
                match op {
                    UnaryOp::Neg => match ty {
                        JvmType::Int | JvmType::Bool => ctx.em.emit_u8(INEG),
                        JvmType::Double => ctx.em.emit_u8(DNEG),
                        _ => {}
                    },
                    UnaryOp::Not | UnaryOp::LogicalNot => {
                        // Logical NOT for int/bool: XOR with 1.
                        if ty != JvmType::Int && ty != JvmType::Bool {
                            self.coerce_stack(ctx, ty, JvmType::Int);
                        }
                        ctx.em.emit_u8(ICONST_1);
                        ctx.push(1);
                        ctx.em.emit_u8(IXOR);
                        ctx.pop(1);
                    }
                    _ => {}
                }
                Ok(ty)
            }

            // ── Function calls ────────────────────────────────────────────────
            Expr::Call(callee, args) => self.compile_call(ctx, callee, args),

            _ => {
                // Unsupported expression — push null.
                ctx.em.emit_u8(ACONST_NULL);
                ctx.push(1);
                Ok(JvmType::Ref)
            }
        }
    }

    // ── Binary operation ──────────────────────────────────────────────────────

    fn compile_binop(
        &mut self,
        ctx: &mut MethodCtx,
        lhs: &Spanned<Expr>,
        op: BinOp,
        rhs: &Spanned<Expr>,
    ) -> Result<JvmType, JvmError> {
        // String + anything or anything + string → StringBuilder concatenation.
        if op == BinOp::Add
            && (matches!(lhs.node, Expr::StringLit(_)) || matches!(rhs.node, Expr::StringLit(_)))
        {
            return self.compile_string_concat(ctx, &lhs.node, &rhs.node);
        }

        let lhs_ty = self.compile_expr(ctx, &lhs.node)?;
        let rhs_ty = self.compile_expr(ctx, &rhs.node)?;

        // Determine result type and promote to Double if needed.
        let use_double = lhs_ty == JvmType::Double || rhs_ty == JvmType::Double;

        if use_double {
            // Promote integers to double.
            // Current stack (top → bottom): rhs, lhs.
            if rhs_ty == JvmType::Int || rhs_ty == JvmType::Bool {
                ctx.em.emit_u8(I2D);
                ctx.push(1); // int→double: 1 slot becomes 2
            }
            if lhs_ty == JvmType::Int || lhs_ty == JvmType::Bool {
                // Swap to get lhs on top, convert, then swap back.
                // double is 2 slots; we can't easily SWAP here.
                // Simpler: store rhs in a temp local, convert lhs, reload.
                // For now fall back to just emitting i2d when lhs is on top
                // before rhs — but the order here has rhs on top.
                // We'll handle this by emitting DUP2/swap tricks or a temp local.
                // Safest approach: we store rhs in a temp double local.
                let tmp_slot = ctx.locals.alloc("$tmp_d", JvmType::Double);
                ctx.em.emit_dstore(tmp_slot);
                ctx.pop(2);
                ctx.em.emit_u8(I2D);
                ctx.pop(1);
                ctx.push(2);
                ctx.em.emit_dload(tmp_slot);
                ctx.push(2);
            }
        }

        // Compute final result type.
        let result_ty = if use_double {
            JvmType::Double
        } else {
            JvmType::Int
        };

        // Operands are on stack (lhs, then rhs on top). Pop both and push result.
        ctx.pop(2 * result_ty.slots() as u16);
        ctx.push(result_ty.slots() as u16);

        match op {
            BinOp::Add => {
                if use_double {
                    ctx.em.emit_u8(DADD)
                } else {
                    ctx.em.emit_u8(IADD)
                }
            }
            BinOp::Sub => {
                if use_double {
                    ctx.em.emit_u8(DSUB)
                } else {
                    ctx.em.emit_u8(ISUB)
                }
            }
            BinOp::Mul => {
                if use_double {
                    ctx.em.emit_u8(DMUL)
                } else {
                    ctx.em.emit_u8(IMUL)
                }
            }
            BinOp::Div | BinOp::FloorDiv => {
                if use_double {
                    ctx.em.emit_u8(DDIV)
                } else {
                    ctx.em.emit_u8(IDIV)
                }
            }
            BinOp::Mod => {
                if use_double {
                    ctx.em.emit_u8(DREM)
                } else {
                    ctx.em.emit_u8(IREM)
                }
            }

            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                // Replace the result slot with a boolean (1 slot).
                ctx.pop(result_ty.slots() as u16);
                ctx.push(1);
                self.compile_comparison(ctx, op, use_double)?;
                return Ok(JvmType::Bool);
            }

            BinOp::And | BinOp::Or | BinOp::BitAnd | BinOp::BitOr => {
                ctx.em.emit_u8(if matches!(op, BinOp::And | BinOp::BitAnd) {
                    IAND
                } else {
                    IOR
                });
            }
            BinOp::BitXor => {
                ctx.em.emit_u8(IXOR);
            }

            _ => {
                // Unsupported op — pop both and push null.
                ctx.em.emit_pop();
                ctx.em.emit_pop();
                ctx.em.emit_u8(ACONST_NULL);
                ctx.pop(result_ty.slots() as u16);
                ctx.push(1);
                return Ok(JvmType::Ref);
            }
        }

        Ok(result_ty)
    }

    /// Emit a comparison that leaves 0 (false) or 1 (true) on the stack.
    /// The two operands are already on the stack and have been popped from
    /// `ctx.cur_stack` by the caller.
    fn compile_comparison(
        &mut self,
        ctx: &mut MethodCtx,
        op: BinOp,
        use_double: bool,
    ) -> Result<(), JvmError> {
        // Pattern:
        //   <cmp opcode>      (pops 2, pushes 0 for IF_ICMP* or pushes int for DCMPG)
        //   <branch opcode>   branch to TRUE label
        //   iconst_0          (false path — fall through)
        //   goto END
        // TRUE:
        //   iconst_1
        // END:

        if use_double {
            // DCMPG: pops 2 doubles, pushes int (-1/0/1).
            ctx.em.emit_u8(DCMPG);
            // Now compare the int result to 0 using IFEQ/IFNE/IFLT/IFGT/IFLE/IFGE.
            let branch_op = match op {
                BinOp::Eq => IFEQ,
                BinOp::NotEq => IFNE,
                BinOp::Lt => IFLT,
                BinOp::Gt => IFGT,
                BinOp::LtEq => IFLE,
                BinOp::GtEq => IFGE,
                _ => IFEQ,
            };
            ctx.em.emit_u8(branch_op);
            let true_branch_pos = ctx.em.position();
            ctx.em.emit_u16(0); // placeholder

            // False path.
            ctx.em.emit_u8(ICONST_0);
            let goto_pos = ctx.em.emit_goto();

            // True path.
            let true_target = ctx.em.position();
            ctx.em.patch_branch(true_branch_pos, true_target);
            ctx.em.emit_u8(ICONST_1);

            // End.
            let end_target = ctx.em.position();
            ctx.em.patch_branch(goto_pos, end_target);
        } else {
            // Integer comparison: IF_ICMP* pops two ints.
            let branch_op = match op {
                BinOp::Eq => IF_ICMPEQ,
                BinOp::NotEq => IF_ICMPNE,
                BinOp::Lt => IF_ICMPLT,
                BinOp::Gt => IF_ICMPGT,
                BinOp::LtEq => IF_ICMPLE,
                BinOp::GtEq => IF_ICMPGE,
                _ => IF_ICMPEQ,
            };
            ctx.em.emit_u8(branch_op);
            let true_branch_pos = ctx.em.position();
            ctx.em.emit_u16(0); // placeholder

            // False path.
            ctx.em.emit_u8(ICONST_0);
            let goto_pos = ctx.em.emit_goto();

            // True path.
            let true_target = ctx.em.position();
            ctx.em.patch_branch(true_branch_pos, true_target);
            ctx.em.emit_u8(ICONST_1);

            // End.
            let end_target = ctx.em.position();
            ctx.em.patch_branch(goto_pos, end_target);
        }
        Ok(())
    }

    // ── String concatenation via StringBuilder ────────────────────────────────

    fn compile_string_concat(
        &mut self,
        ctx: &mut MethodCtx,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<JvmType, JvmError> {
        let sb_class = ctx.cp.class("java/lang/StringBuilder");
        let sb_init = ctx
            .cp
            .method_ref("java/lang/StringBuilder", "<init>", "()V");
        let sb_append_str = ctx.cp.method_ref(
            "java/lang/StringBuilder",
            "append",
            "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
        );
        let sb_append_obj = ctx.cp.method_ref(
            "java/lang/StringBuilder",
            "append",
            "(Ljava/lang/Object;)Ljava/lang/StringBuilder;",
        );
        let sb_tostring = ctx.cp.method_ref(
            "java/lang/StringBuilder",
            "toString",
            "()Ljava/lang/String;",
        );

        // new StringBuilder()
        ctx.em.emit_new(sb_class);
        ctx.em.emit_dup();
        ctx.push(2);
        ctx.em.emit_invokespecial(sb_init);
        ctx.pop(1); // invokespecial pops the dup'd ref

        // Append lhs.
        let lhs_ty = self.compile_expr(ctx, lhs)?;
        let lhs_append = if lhs_ty == JvmType::Ref {
            sb_append_str
        } else {
            sb_append_obj
        };
        ctx.em.emit_invokevirtual(lhs_append);
        ctx.pop(1 + lhs_ty.slots() as u16); // pops sb + lhs, pushes sb
        ctx.push(1);

        // Append rhs.
        let rhs_ty = self.compile_expr(ctx, rhs)?;
        let rhs_append = if rhs_ty == JvmType::Ref {
            sb_append_str
        } else {
            sb_append_obj
        };
        ctx.em.emit_invokevirtual(rhs_append);
        ctx.pop(1 + rhs_ty.slots() as u16);
        ctx.push(1);

        // toString()
        ctx.em.emit_invokevirtual(sb_tostring);
        // Stack: sb ref → String ref (1 slot in, 1 slot out — no net change).
        Ok(JvmType::Ref)
    }

    // ── Call compilation ──────────────────────────────────────────────────────

    fn compile_call(
        &mut self,
        ctx: &mut MethodCtx,
        callee: &Spanned<Expr>,
        args: &[Argument],
    ) -> Result<JvmType, JvmError> {
        if let Expr::Ident(name) = &callee.node {
            if name == "print" {
                return self.compile_print(ctx, args);
            }
            // Call a static method in the same class (best-effort descriptor).
            let mut desc = String::from("(");
            let mut arg_tys: Vec<JvmType> = Vec::new();
            for arg in args {
                let ty = self.compile_expr(ctx, &arg.value.node)?;
                desc.push_str(ty.descriptor());
                arg_tys.push(ty);
            }
            desc.push_str(")Ljava/lang/Object;");
            let class_name = self.class_name.clone();
            let mref = ctx.cp.method_ref(&class_name, name, &desc);
            ctx.em.emit_invokestatic(mref);
            for ty in &arg_tys {
                ctx.pop(ty.slots() as u16);
            }
            ctx.push(1); // return value (Object)
            return Ok(JvmType::Ref);
        }

        // Unsupported callee — push null.
        ctx.em.emit_u8(ACONST_NULL);
        ctx.push(1);
        Ok(JvmType::Ref)
    }

    /// Compile a `print(...)` call as `System.out.println(...)`.
    fn compile_print(
        &mut self,
        ctx: &mut MethodCtx,
        args: &[Argument],
    ) -> Result<JvmType, JvmError> {
        let out_field = ctx
            .cp
            .field_ref("java/lang/System", "out", "Ljava/io/PrintStream;");
        let println_str =
            ctx.cp
                .method_ref("java/io/PrintStream", "println", "(Ljava/lang/String;)V");
        let println_obj =
            ctx.cp
                .method_ref("java/io/PrintStream", "println", "(Ljava/lang/Object;)V");
        let println_int = ctx.cp.method_ref("java/io/PrintStream", "println", "(I)V");
        let println_dbl = ctx.cp.method_ref("java/io/PrintStream", "println", "(D)V");
        let println_noarg = ctx.cp.method_ref("java/io/PrintStream", "println", "()V");

        ctx.em.emit_getstatic(out_field);
        ctx.push(1);

        let arg_ty = if args.is_empty() {
            ctx.em.emit_invokevirtual(println_noarg);
            ctx.pop(1);
            // push null so Stmt::Expr's POP works.
            ctx.em.emit_u8(ACONST_NULL);
            ctx.push(1);
            return Ok(JvmType::Ref);
        } else {
            self.compile_expr(ctx, &args[0].value.node)?
        };

        let println_ref = match arg_ty {
            JvmType::Int | JvmType::Bool => println_int,
            JvmType::Double => println_dbl,
            JvmType::Ref => println_str,
            JvmType::Void => {
                ctx.em.emit_u8(ACONST_NULL);
                ctx.push(1);
                println_obj
            }
        };
        ctx.em.emit_invokevirtual(println_ref);
        ctx.pop(1 + arg_ty.slots() as u16); // pops System.out ref + arg

        // Push null so Stmt::Expr's POP does not underflow.
        ctx.em.emit_u8(ACONST_NULL);
        ctx.push(1);
        Ok(JvmType::Ref)
    }

    // ── if / else ─────────────────────────────────────────────────────────────

    fn compile_if(&mut self, ctx: &mut MethodCtx, if_stmt: &IfStmt) -> Result<(), JvmError> {
        let cond_ty = self.compile_expr(ctx, &if_stmt.condition.node)?;
        ctx.pop(cond_ty.slots() as u16);

        // IFEQ: branch to else/end when condition is 0 (false).
        ctx.em.emit_u8(IFEQ);
        let false_pos = ctx.em.position();
        ctx.em.emit_u16(0);

        for s in &if_stmt.then_block.statements {
            self.compile_stmt(ctx, &s.node)?;
        }

        if if_stmt.else_block.is_some() || !if_stmt.elif_clauses.is_empty() {
            let goto_pos = ctx.em.emit_goto();
            let else_start = ctx.em.position();
            ctx.em.patch_branch(false_pos, else_start);

            if let Some(else_block) = &if_stmt.else_block {
                for s in &else_block.statements {
                    self.compile_stmt(ctx, &s.node)?;
                }
            }

            let end = ctx.em.position();
            ctx.em.patch_branch(goto_pos, end);
        } else {
            let end = ctx.em.position();
            ctx.em.patch_branch(false_pos, end);
        }
        Ok(())
    }

    // ── while loop ────────────────────────────────────────────────────────────

    fn compile_while(&mut self, ctx: &mut MethodCtx, ws: &WhileStmt) -> Result<(), JvmError> {
        let loop_start = ctx.em.position();

        let cond_ty = self.compile_expr(ctx, &ws.condition.node)?;
        ctx.pop(cond_ty.slots() as u16);

        // IFEQ → exit loop when false.
        ctx.em.emit_u8(IFEQ);
        let exit_pos = ctx.em.position();
        ctx.em.emit_u16(0);

        for s in &ws.body.statements {
            self.compile_stmt(ctx, &s.node)?;
        }

        // GOTO back to loop_start.
        ctx.em.emit_u8(GOTO);
        let back_pos = ctx.em.position();
        ctx.em.emit_u16(0);
        ctx.em.patch_branch(back_pos, loop_start);

        let loop_end = ctx.em.position();
        ctx.em.patch_branch(exit_pos, loop_end);
        Ok(())
    }

    // ── Type coercion on the stack ────────────────────────────────────────────

    /// Emit conversion instructions to change the top-of-stack from `from` to `to`.
    fn coerce_stack(&mut self, ctx: &mut MethodCtx, from: JvmType, to: JvmType) {
        match (from, to) {
            (JvmType::Int, JvmType::Double) | (JvmType::Bool, JvmType::Double) => {
                ctx.em.emit_u8(I2D);
                ctx.push(1); // int (1 slot) → double (2 slots)
            }
            (JvmType::Double, JvmType::Int) | (JvmType::Double, JvmType::Bool) => {
                ctx.em.emit_u8(D2I);
                ctx.pop(1); // double (2 slots) → int (1 slot)
            }
            _ => {} // no conversion needed
        }
    }

    // ── Method info builder ───────────────────────────────────────────────────

    fn build_method(
        &mut self,
        access_flags: u16,
        name: &str,
        descriptor: &str,
        code: &[u8],
        max_stack: u16,
        max_locals: u16,
    ) -> Vec<u8> {
        let name_idx = self.cp.utf8(name);
        let desc_idx = self.cp.utf8(descriptor);
        let code_attr_idx = self.cp.utf8("Code");

        let mut buf = Vec::new();
        buf.extend_from_slice(&access_flags.to_be_bytes());
        buf.extend_from_slice(&name_idx.to_be_bytes());
        buf.extend_from_slice(&desc_idx.to_be_bytes());
        buf.extend_from_slice(&1u16.to_be_bytes()); // attributes_count = 1

        let code_attr = Emitter::build_code_attribute(code, max_stack, max_locals, code_attr_idx);
        buf.extend_from_slice(&code_attr);

        buf
    }
}

// ── Public convenience function ───────────────────────────────────────────────

/// Parse `source` as UniLang and compile it to a JVM `.class` file.
///
/// `class_name` should be a valid Java binary name such as `"Hello"` or
/// `"com/example/MyClass"`.  Returns the raw bytes of the `.class` file.
///
/// # Errors
/// Returns a human-readable string if parsing or compilation fails.
pub fn unilang_compile_to_jvm(source: &str, class_name: &str) -> Result<Vec<u8>, String> {
    use unilang_common::source::SourceMap;

    let mut sm = SourceMap::new();
    let sid = sm.add("<jvm-input>".to_string(), source.to_string());

    let (module, diags) = unilang_parser::parse(sid, source);
    if diags.has_errors() {
        return Err("parse errors in source".to_string());
    }

    JvmCompiler::compile(&module, class_name).map_err(|e| e.to_string())
}
