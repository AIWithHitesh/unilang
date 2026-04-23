// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Low-level JVM bytecode emission helpers.
//!
//! An `Emitter` holds a `Vec<u8>` code buffer and exposes small methods for
//! emitting individual instructions.  All branch targets are expressed as
//! absolute byte offsets into the buffer; use `patch_branch` to back-fill
//! them once the target is known.

use crate::opcodes::*;

/// Code buffer for a single method body.
#[derive(Debug, Default)]
pub struct Emitter {
    code: Vec<u8>,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the current write position (used as a branch target).
    pub fn position(&self) -> usize {
        self.code.len()
    }

    /// Consume the emitter and return the raw bytecode.
    pub fn into_bytes(self) -> Vec<u8> {
        self.code
    }

    // ── Raw emission ─────────────────────────────────────────────────────────

    pub fn emit_u8(&mut self, b: u8) {
        self.code.push(b);
    }

    pub fn emit_u16(&mut self, v: u16) {
        self.code.extend_from_slice(&v.to_be_bytes());
    }

    pub fn emit_u32(&mut self, v: u32) {
        self.code.extend_from_slice(&v.to_be_bytes());
    }

    pub fn emit_i16(&mut self, v: i16) {
        self.code.extend_from_slice(&v.to_be_bytes());
    }

    // ── Constants ────────────────────────────────────────────────────────────

    /// Push a small integer constant (uses iconst_<n> for -1..=5, bipush for
    /// -128..=127, sipush for -32768..=32767, or ldc otherwise).
    pub fn emit_int_const(&mut self, v: i32, cp_index: Option<u16>) {
        match v {
            -1 => self.emit_u8(ICONST_M1),
            0 => self.emit_u8(ICONST_0),
            1 => self.emit_u8(ICONST_1),
            2 => self.emit_u8(ICONST_2),
            3 => self.emit_u8(ICONST_3),
            4 => self.emit_u8(ICONST_4),
            5 => self.emit_u8(ICONST_5),
            -128..=127 => {
                self.emit_u8(BIPUSH);
                self.emit_u8(v as u8);
            }
            -32768..=32767 => {
                self.emit_u8(SIPUSH);
                self.emit_i16(v as i16);
            }
            _ => {
                if let Some(idx) = cp_index {
                    self.emit_ldc_w(idx);
                }
            }
        }
    }

    /// Push a long constant.
    pub fn emit_long_const(&mut self, v: i64) {
        match v {
            0 => self.emit_u8(LCONST_0),
            1 => self.emit_u8(LCONST_1),
            _ => {
                // Caller must pass a valid wide cp index for ldc2_w
            }
        }
    }

    /// Push a double constant.
    pub fn emit_double_const(&mut self, v: f64) {
        if v == 0.0 {
            self.emit_u8(DCONST_0);
        } else if v == 1.0 {
            self.emit_u8(DCONST_1);
        }
        // Other values require ldc2_w — use emit_ldc2_w directly.
    }

    // ── LDC variants ─────────────────────────────────────────────────────────

    /// ldc — 1-byte cp index (fits in 0..=255).
    pub fn emit_ldc(&mut self, cp_index: u16) {
        if cp_index <= 255 {
            self.emit_u8(LDC);
            self.emit_u8(cp_index as u8);
        } else {
            self.emit_ldc_w(cp_index);
        }
    }

    /// ldc_w — 2-byte cp index.
    pub fn emit_ldc_w(&mut self, cp_index: u16) {
        self.emit_u8(LDC_W);
        self.emit_u16(cp_index);
    }

    /// ldc2_w — 2-byte cp index for Long / Double.
    pub fn emit_ldc2_w(&mut self, cp_index: u16) {
        self.emit_u8(LDC2_W);
        self.emit_u16(cp_index);
    }

    // ── Loads ─────────────────────────────────────────────────────────────────

    pub fn emit_iload(&mut self, slot: u8) {
        match slot {
            0 => self.emit_u8(ILOAD_0),
            1 => self.emit_u8(ILOAD_1),
            2 => self.emit_u8(ILOAD_2),
            3 => self.emit_u8(ILOAD_3),
            _ => {
                self.emit_u8(ILOAD);
                self.emit_u8(slot);
            }
        }
    }

    pub fn emit_dload(&mut self, slot: u8) {
        match slot {
            0 => self.emit_u8(DLOAD_0),
            _ => {
                self.emit_u8(DLOAD);
                self.emit_u8(slot);
            }
        }
    }

    pub fn emit_aload(&mut self, slot: u8) {
        match slot {
            0 => self.emit_u8(ALOAD_0),
            1 => self.emit_u8(ALOAD_1),
            2 => self.emit_u8(ALOAD_2),
            3 => self.emit_u8(ALOAD_3),
            _ => {
                self.emit_u8(ALOAD);
                self.emit_u8(slot);
            }
        }
    }

    // ── Stores ────────────────────────────────────────────────────────────────

    pub fn emit_istore(&mut self, slot: u8) {
        match slot {
            0 => self.emit_u8(ISTORE_0),
            1 => self.emit_u8(ISTORE_1),
            2 => self.emit_u8(ISTORE_2),
            3 => self.emit_u8(ISTORE_3),
            _ => {
                self.emit_u8(ISTORE);
                self.emit_u8(slot);
            }
        }
    }

    pub fn emit_dstore(&mut self, slot: u8) {
        self.emit_u8(DSTORE);
        self.emit_u8(slot);
    }

    pub fn emit_astore(&mut self, slot: u8) {
        match slot {
            0 => self.emit_u8(ASTORE_0),
            1 => self.emit_u8(ASTORE_1),
            2 => self.emit_u8(ASTORE_2),
            3 => self.emit_u8(ASTORE_3),
            _ => {
                self.emit_u8(ASTORE);
                self.emit_u8(slot);
            }
        }
    }

    // ── Arithmetic ────────────────────────────────────────────────────────────

    pub fn emit_iadd(&mut self) {
        self.emit_u8(IADD);
    }
    pub fn emit_isub(&mut self) {
        self.emit_u8(ISUB);
    }
    pub fn emit_imul(&mut self) {
        self.emit_u8(IMUL);
    }
    pub fn emit_idiv(&mut self) {
        self.emit_u8(IDIV);
    }
    pub fn emit_irem(&mut self) {
        self.emit_u8(IREM);
    }
    pub fn emit_ineg(&mut self) {
        self.emit_u8(INEG);
    }

    pub fn emit_dadd(&mut self) {
        self.emit_u8(DADD);
    }
    pub fn emit_dsub(&mut self) {
        self.emit_u8(DSUB);
    }
    pub fn emit_dmul(&mut self) {
        self.emit_u8(DMUL);
    }
    pub fn emit_ddiv(&mut self) {
        self.emit_u8(DDIV);
    }
    pub fn emit_dneg(&mut self) {
        self.emit_u8(DNEG);
    }

    // ── Conversions ───────────────────────────────────────────────────────────

    pub fn emit_i2d(&mut self) {
        self.emit_u8(I2D);
    }
    pub fn emit_d2i(&mut self) {
        self.emit_u8(D2I);
    }

    // ── Stack ops ─────────────────────────────────────────────────────────────

    pub fn emit_pop(&mut self) {
        self.emit_u8(POP);
    }
    pub fn emit_dup(&mut self) {
        self.emit_u8(DUP);
    }
    pub fn emit_dup_x1(&mut self) {
        self.emit_u8(DUP_X1);
    }

    // ── Field / method calls ──────────────────────────────────────────────────

    pub fn emit_getstatic(&mut self, cp_index: u16) {
        self.emit_u8(GETSTATIC);
        self.emit_u16(cp_index);
    }

    pub fn emit_invokevirtual(&mut self, cp_index: u16) {
        self.emit_u8(INVOKEVIRTUAL);
        self.emit_u16(cp_index);
    }

    pub fn emit_invokespecial(&mut self, cp_index: u16) {
        self.emit_u8(INVOKESPECIAL);
        self.emit_u16(cp_index);
    }

    pub fn emit_invokestatic(&mut self, cp_index: u16) {
        self.emit_u8(INVOKESTATIC);
        self.emit_u16(cp_index);
    }

    pub fn emit_new(&mut self, cp_index: u16) {
        self.emit_u8(NEW);
        self.emit_u16(cp_index);
    }

    // ── Returns ───────────────────────────────────────────────────────────────

    pub fn emit_return(&mut self) {
        self.emit_u8(RETURN);
    }
    pub fn emit_ireturn(&mut self) {
        self.emit_u8(IRETURN);
    }
    pub fn emit_dreturn(&mut self) {
        self.emit_u8(DRETURN);
    }
    pub fn emit_areturn(&mut self) {
        self.emit_u8(ARETURN);
    }

    // ── Comparisons / branches ────────────────────────────────────────────────

    /// Emit a GOTO.  Returns the position of the branch offset bytes for
    /// `patch_branch`.
    pub fn emit_goto(&mut self) -> usize {
        self.emit_u8(GOTO);
        let patch_pos = self.position();
        self.emit_u16(0); // placeholder
        patch_pos
    }

    /// Emit IF_ICMPEQ (branch if top two ints equal).
    pub fn emit_if_icmpeq(&mut self) -> usize {
        self.emit_u8(IF_ICMPEQ);
        let patch_pos = self.position();
        self.emit_u16(0);
        patch_pos
    }

    /// Emit IFEQ (branch if top int == 0).
    pub fn emit_ifeq(&mut self) -> usize {
        self.emit_u8(IFEQ);
        let patch_pos = self.position();
        self.emit_u16(0);
        patch_pos
    }

    /// Emit IFNE.
    pub fn emit_ifne(&mut self) -> usize {
        self.emit_u8(IFNE);
        let patch_pos = self.position();
        self.emit_u16(0);
        patch_pos
    }

    /// Emit DCMPG then IFEQ (double comparison equal).
    pub fn emit_dcmpeq(&mut self) -> usize {
        self.emit_u8(DCMPG);
        self.emit_ifeq()
    }

    /// Patch a previously emitted branch instruction.
    ///
    /// `patch_pos` is the byte position returned by an `emit_*` branch method.
    /// `target` is the absolute byte offset of the branch destination.
    /// The JVM branch offset is relative to the start of the branch instruction
    /// (which is 1 byte before `patch_pos` for all our 3-byte branch opcodes).
    pub fn patch_branch(&mut self, patch_pos: usize, target: usize) {
        let instr_start = patch_pos - 1;
        let offset = target as i64 - instr_start as i64;
        let offset_i16 = offset as i16;
        let bytes = offset_i16.to_be_bytes();
        self.code[patch_pos] = bytes[0];
        self.code[patch_pos + 1] = bytes[1];
    }

    // ── Attribute building helpers ────────────────────────────────────────────

    /// Build a "Code" attribute body.
    ///
    /// Returns the raw bytes of the Code attribute *value* (not including the
    /// attribute_name_index or attribute_length — those are written by the
    /// caller).
    pub fn build_code_attribute(
        code: &[u8],
        max_stack: u16,
        max_locals: u16,
        cp_code_attr_index: u16,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        // attribute_name_index
        buf.extend_from_slice(&cp_code_attr_index.to_be_bytes());
        // attribute_length (will be patched below)
        let length_pos = buf.len();
        buf.extend_from_slice(&0u32.to_be_bytes());

        // max_stack
        buf.extend_from_slice(&max_stack.to_be_bytes());
        // max_locals
        buf.extend_from_slice(&max_locals.to_be_bytes());
        // code_length
        buf.extend_from_slice(&(code.len() as u32).to_be_bytes());
        // code
        buf.extend_from_slice(code);
        // exception_table_length
        buf.extend_from_slice(&0u16.to_be_bytes());
        // attributes_count
        buf.extend_from_slice(&0u16.to_be_bytes());

        // Back-patch attribute_length.
        let attr_len = (buf.len() - length_pos - 4) as u32;
        let bytes = attr_len.to_be_bytes();
        buf[length_pos] = bytes[0];
        buf[length_pos + 1] = bytes[1];
        buf[length_pos + 2] = bytes[2];
        buf[length_pos + 3] = bytes[3];

        buf
    }
}
