// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! JVM constant pool builder.
//!
//! The JVM constant pool is a 1-indexed array.  Index 0 is unused and indices
//! for `Long` and `Double` entries consume two slots (the second slot is
//! unusable), so we track that here transparently.

use crate::opcodes::*;

/// A single entry in the JVM constant pool.
#[derive(Debug, Clone)]
pub enum CpEntry {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name_index: u16,
    },
    StringRef {
        utf8_index: u16,
    },
    FieldRef {
        class_index: u16,
        nat_index: u16,
    },
    MethodRef {
        class_index: u16,
        nat_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        nat_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    /// Placeholder used as the "second slot" after a Long or Double.
    LargeSlotPad,
}

impl CpEntry {
    fn tag(&self) -> u8 {
        match self {
            CpEntry::Utf8(_) => CONSTANT_UTF8,
            CpEntry::Integer(_) => CONSTANT_INTEGER,
            CpEntry::Float(_) => CONSTANT_FLOAT,
            CpEntry::Long(_) => CONSTANT_LONG,
            CpEntry::Double(_) => CONSTANT_DOUBLE,
            CpEntry::Class { .. } => CONSTANT_CLASS,
            CpEntry::StringRef { .. } => CONSTANT_STRING,
            CpEntry::FieldRef { .. } => CONSTANT_FIELDREF,
            CpEntry::MethodRef { .. } => CONSTANT_METHODREF,
            CpEntry::InterfaceMethodRef { .. } => CONSTANT_INTERFACE_METHODREF,
            CpEntry::NameAndType { .. } => CONSTANT_NAME_AND_TYPE,
            CpEntry::LargeSlotPad => unreachable!("LargeSlotPad should never be written directly"),
        }
    }

    /// Serialize this entry to JVM class-file bytes.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        match self {
            CpEntry::LargeSlotPad => {} // second slot of a Long/Double — not emitted
            CpEntry::Utf8(s) => {
                out.push(self.tag());
                let bytes = s.as_bytes();
                out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                out.extend_from_slice(bytes);
            }
            CpEntry::Integer(v) => {
                out.push(self.tag());
                out.extend_from_slice(&v.to_be_bytes());
            }
            CpEntry::Float(v) => {
                out.push(self.tag());
                out.extend_from_slice(&v.to_bits().to_be_bytes());
            }
            CpEntry::Long(v) => {
                out.push(self.tag());
                out.extend_from_slice(&v.to_be_bytes());
            }
            CpEntry::Double(v) => {
                out.push(self.tag());
                out.extend_from_slice(&v.to_bits().to_be_bytes());
            }
            CpEntry::Class { name_index } => {
                out.push(self.tag());
                out.extend_from_slice(&name_index.to_be_bytes());
            }
            CpEntry::StringRef { utf8_index } => {
                out.push(self.tag());
                out.extend_from_slice(&utf8_index.to_be_bytes());
            }
            CpEntry::FieldRef {
                class_index,
                nat_index,
            }
            | CpEntry::MethodRef {
                class_index,
                nat_index,
            }
            | CpEntry::InterfaceMethodRef {
                class_index,
                nat_index,
            } => {
                out.push(self.tag());
                out.extend_from_slice(&class_index.to_be_bytes());
                out.extend_from_slice(&nat_index.to_be_bytes());
            }
            CpEntry::NameAndType {
                name_index,
                descriptor_index,
            } => {
                out.push(self.tag());
                out.extend_from_slice(&name_index.to_be_bytes());
                out.extend_from_slice(&descriptor_index.to_be_bytes());
            }
        }
    }
}

/// Builder for the JVM constant pool.
///
/// Entries are de-duplicated by content; you always get back a stable 1-based
/// index.
#[derive(Debug, Default)]
pub struct ConstantPool {
    /// The internal list (0-indexed here, +1 = JVM index).
    entries: Vec<CpEntry>,
}

impl ConstantPool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of JVM constant-pool slots consumed (the count stored in the
    /// class file header is this value + 1).
    pub fn len(&self) -> u16 {
        self.entries.len() as u16
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize all entries in pool order.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        for e in &self.entries {
            e.write_to(out);
        }
    }

    // ── Core insertion helpers ────────────────────────────────────────────────

    /// Insert `entry` unconditionally and return its 1-based JVM index.
    fn push(&mut self, entry: CpEntry) -> u16 {
        let idx = self.entries.len() as u16 + 1; // 1-based
        let large = matches!(entry, CpEntry::Long(_) | CpEntry::Double(_));
        self.entries.push(entry);
        if large {
            self.entries.push(CpEntry::LargeSlotPad);
        }
        idx
    }

    // ── Public builder methods ────────────────────────────────────────────────

    /// Add a UTF-8 string (or reuse an existing one).
    pub fn utf8(&mut self, s: &str) -> u16 {
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::Utf8(existing) = e {
                if existing == s {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::Utf8(s.to_string()))
    }

    /// Add a Class reference.
    pub fn class(&mut self, binary_name: &str) -> u16 {
        let name_index = self.utf8(binary_name);
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::Class { name_index: ni } = e {
                if *ni == name_index {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::Class { name_index })
    }

    /// Add a String constant reference.
    pub fn string(&mut self, s: &str) -> u16 {
        let utf8_index = self.utf8(s);
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::StringRef { utf8_index: ui } = e {
                if *ui == utf8_index {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::StringRef { utf8_index })
    }

    /// Add an integer constant.
    pub fn integer(&mut self, v: i32) -> u16 {
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::Integer(ev) = e {
                if *ev == v {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::Integer(v))
    }

    /// Add a float constant.
    pub fn float(&mut self, v: f32) -> u16 {
        let bits = v.to_bits();
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::Float(ev) = e {
                if ev.to_bits() == bits {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::Float(v))
    }

    /// Add a long constant.
    pub fn long(&mut self, v: i64) -> u16 {
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::Long(ev) = e {
                if *ev == v {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::Long(v))
    }

    /// Add a double constant.
    pub fn double(&mut self, v: f64) -> u16 {
        let bits = v.to_bits();
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::Double(ev) = e {
                if ev.to_bits() == bits {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::Double(v))
    }

    /// Add a NameAndType entry.
    pub fn name_and_type(&mut self, name: &str, descriptor: &str) -> u16 {
        let name_index = self.utf8(name);
        let descriptor_index = self.utf8(descriptor);
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::NameAndType {
                name_index: ni,
                descriptor_index: di,
            } = e
            {
                if *ni == name_index && *di == descriptor_index {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::NameAndType {
            name_index,
            descriptor_index,
        })
    }

    /// Add a FieldRef entry.
    pub fn field_ref(&mut self, class: &str, name: &str, descriptor: &str) -> u16 {
        let class_index = self.class(class);
        let nat_index = self.name_and_type(name, descriptor);
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::FieldRef {
                class_index: ci,
                nat_index: ni,
            } = e
            {
                if *ci == class_index && *ni == nat_index {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::FieldRef {
            class_index,
            nat_index,
        })
    }

    /// Add a MethodRef entry.
    pub fn method_ref(&mut self, class: &str, name: &str, descriptor: &str) -> u16 {
        let class_index = self.class(class);
        let nat_index = self.name_and_type(name, descriptor);
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::MethodRef {
                class_index: ci,
                nat_index: ni,
            } = e
            {
                if *ci == class_index && *ni == nat_index {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::MethodRef {
            class_index,
            nat_index,
        })
    }

    /// Add an InterfaceMethodRef entry.
    pub fn interface_method_ref(&mut self, class: &str, name: &str, descriptor: &str) -> u16 {
        let class_index = self.class(class);
        let nat_index = self.name_and_type(name, descriptor);
        for (i, e) in self.entries.iter().enumerate() {
            if let CpEntry::InterfaceMethodRef {
                class_index: ci,
                nat_index: ni,
            } = e
            {
                if *ci == class_index && *ni == nat_index {
                    return i as u16 + 1;
                }
            }
        }
        self.push(CpEntry::InterfaceMethodRef {
            class_index,
            nat_index,
        })
    }
}
