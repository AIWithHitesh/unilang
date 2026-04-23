// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! JVM opcode constants.
//!
//! Only the subset needed to compile basic UniLang programs is included here.
//! See: <https://docs.oracle.com/javase/specs/jvms/se11/html/jvms-6.html>

// ── Stack / Load / Store ──────────────────────────────────────────────────────

/// Push the null reference.
pub const ACONST_NULL: u8 = 0x01;
/// Push int constant -1.
pub const ICONST_M1: u8 = 0x02;
/// Push int constant 0.
pub const ICONST_0: u8 = 0x03;
/// Push int constant 1.
pub const ICONST_1: u8 = 0x04;
/// Push int constant 2.
pub const ICONST_2: u8 = 0x05;
/// Push int constant 3.
pub const ICONST_3: u8 = 0x06;
/// Push int constant 4.
pub const ICONST_4: u8 = 0x07;
/// Push int constant 5.
pub const ICONST_5: u8 = 0x08;
/// Push long constant 0.
pub const LCONST_0: u8 = 0x09;
/// Push long constant 1.
pub const LCONST_1: u8 = 0x0a;
/// Push double constant 0.0.
pub const DCONST_0: u8 = 0x0e;
/// Push double constant 1.0.
pub const DCONST_1: u8 = 0x0f;
/// Push byte as int.
pub const BIPUSH: u8 = 0x10;
/// Push short as int.
pub const SIPUSH: u8 = 0x11;
/// Push item from constant pool (index 1 byte).
pub const LDC: u8 = 0x12;
/// Push item from constant pool (wide index 2 bytes).
pub const LDC_W: u8 = 0x13;
/// Push long or double from constant pool (wide index 2 bytes).
pub const LDC2_W: u8 = 0x14;

// ── Integer loads ─────────────────────────────────────────────────────────────
pub const ILOAD: u8 = 0x15;
pub const LLOAD: u8 = 0x16;
pub const DLOAD: u8 = 0x18;
pub const ALOAD: u8 = 0x19;
pub const ILOAD_0: u8 = 0x1a;
pub const ILOAD_1: u8 = 0x1b;
pub const ILOAD_2: u8 = 0x1c;
pub const ILOAD_3: u8 = 0x1d;
pub const LLOAD_0: u8 = 0x1e;
pub const DLOAD_0: u8 = 0x26;
pub const ALOAD_0: u8 = 0x2a;
pub const ALOAD_1: u8 = 0x2b;
pub const ALOAD_2: u8 = 0x2c;
pub const ALOAD_3: u8 = 0x2d;

// ── Integer stores ────────────────────────────────────────────────────────────
pub const ISTORE: u8 = 0x36;
pub const LSTORE: u8 = 0x37;
pub const DSTORE: u8 = 0x39;
pub const ASTORE: u8 = 0x3a;
pub const ISTORE_0: u8 = 0x3b;
pub const ISTORE_1: u8 = 0x3c;
pub const ISTORE_2: u8 = 0x3d;
pub const ISTORE_3: u8 = 0x3e;
pub const ASTORE_0: u8 = 0x4b;
pub const ASTORE_1: u8 = 0x4c;
pub const ASTORE_2: u8 = 0x4d;
pub const ASTORE_3: u8 = 0x4e;

// ── Stack operations ──────────────────────────────────────────────────────────
pub const POP: u8 = 0x57;
/// Pop the top one or two category-2 values (long/double) or two category-1 values.
pub const POP2: u8 = 0x58;
pub const DUP: u8 = 0x59;
pub const DUP_X1: u8 = 0x5a;
/// Duplicate the top one or two category-2 values.
pub const DUP2: u8 = 0x5c;
pub const SWAP: u8 = 0x5f;

// ── Integer arithmetic ────────────────────────────────────────────────────────
pub const IADD: u8 = 0x60;
pub const LADD: u8 = 0x61;
pub const DADD: u8 = 0x63;
pub const ISUB: u8 = 0x64;
pub const LSUB: u8 = 0x65;
pub const DSUB: u8 = 0x67;
pub const IMUL: u8 = 0x68;
pub const LMUL: u8 = 0x69;
pub const DMUL: u8 = 0x6b;
pub const IDIV: u8 = 0x6c;
pub const LDIV: u8 = 0x6d;
pub const DDIV: u8 = 0x6f;
pub const IREM: u8 = 0x70;
pub const LREM: u8 = 0x71;
pub const DREM: u8 = 0x73;
pub const INEG: u8 = 0x74;
pub const LNEG: u8 = 0x75;
pub const DNEG: u8 = 0x77;

// ── Conversions ───────────────────────────────────────────────────────────────
pub const I2L: u8 = 0x85;
pub const I2D: u8 = 0x87;
pub const L2I: u8 = 0x88;
pub const L2D: u8 = 0x8a;
pub const D2I: u8 = 0x8e;
pub const D2L: u8 = 0x8f;
pub const I2B: u8 = 0x91;
pub const I2S: u8 = 0x93;

// ── Comparisons ───────────────────────────────────────────────────────────────
pub const DCMPL: u8 = 0x97;
pub const DCMPG: u8 = 0x98;
pub const IFEQ: u8 = 0x99;
pub const IFNE: u8 = 0x9a;
pub const IFLT: u8 = 0x9b;
pub const IFGE: u8 = 0x9c;
pub const IFGT: u8 = 0x9d;
pub const IFLE: u8 = 0x9e;
pub const IF_ICMPEQ: u8 = 0x9f;
pub const IF_ICMPNE: u8 = 0xa0;
pub const IF_ICMPLT: u8 = 0xa1;
pub const IF_ICMPGE: u8 = 0xa2;
pub const IF_ICMPGT: u8 = 0xa3;
pub const IF_ICMPLE: u8 = 0xa4;
pub const IF_ACMPEQ: u8 = 0xa5;
pub const IF_ACMPNE: u8 = 0xa6;

// ── Control flow ──────────────────────────────────────────────────────────────
pub const GOTO: u8 = 0xa7;
pub const GOTO_W: u8 = 0xc8;
pub const IFNULL: u8 = 0xc6;
pub const IFNONNULL: u8 = 0xc7;

// ── Returns ───────────────────────────────────────────────────────────────────
pub const IRETURN: u8 = 0xac;
pub const LRETURN: u8 = 0xad;
pub const DRETURN: u8 = 0xaf;
pub const ARETURN: u8 = 0xb0;
pub const RETURN: u8 = 0xb1;

// ── Field / method access ─────────────────────────────────────────────────────
pub const GETSTATIC: u8 = 0xb2;
pub const PUTSTATIC: u8 = 0xb3;
pub const GETFIELD: u8 = 0xb4;
pub const PUTFIELD: u8 = 0xb5;
pub const INVOKEVIRTUAL: u8 = 0xb6;
pub const INVOKESPECIAL: u8 = 0xb7;
pub const INVOKESTATIC: u8 = 0xb8;
pub const INVOKEINTERFACE: u8 = 0xb9;
pub const INVOKEDYNAMIC: u8 = 0xba;

// ── Object creation ───────────────────────────────────────────────────────────
pub const NEW: u8 = 0xbb;
pub const NEWARRAY: u8 = 0xbc;
pub const ANEWARRAY: u8 = 0xbd;
pub const ARRAYLENGTH: u8 = 0xbe;
pub const ATHROW: u8 = 0xbf;
pub const CHECKCAST: u8 = 0xc0;
pub const INSTANCEOF: u8 = 0xc1;

// ── Wide prefix ───────────────────────────────────────────────────────────────
pub const WIDE: u8 = 0xc4;

// ── Constant-pool tag bytes ───────────────────────────────────────────────────
pub const CONSTANT_UTF8: u8 = 1;
pub const CONSTANT_INTEGER: u8 = 3;
pub const CONSTANT_FLOAT: u8 = 4;
pub const CONSTANT_LONG: u8 = 5;
pub const CONSTANT_DOUBLE: u8 = 6;
pub const CONSTANT_CLASS: u8 = 7;
pub const CONSTANT_STRING: u8 = 8;
pub const CONSTANT_FIELDREF: u8 = 9;
pub const CONSTANT_METHODREF: u8 = 10;
pub const CONSTANT_INTERFACE_METHODREF: u8 = 11;
pub const CONSTANT_NAME_AND_TYPE: u8 = 12;
pub const CONSTANT_METHOD_HANDLE: u8 = 15;
pub const CONSTANT_METHOD_TYPE: u8 = 16;
pub const CONSTANT_INVOKE_DYNAMIC: u8 = 18;

// ── Access flags ─────────────────────────────────────────────────────────────
pub const ACC_PUBLIC: u16 = 0x0001;
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_SUPER: u16 = 0x0020;
pub const ACC_SYNCHRONIZED: u16 = 0x0020;
pub const ACC_ABSTRACT: u16 = 0x0400;
