// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Execution limits for the UniLang VM.
//!
//! Applying limits prevents runaway programs from consuming
//! unbounded CPU time, call stack, or memory.

/// Configurable execution limits for a VM instance.
#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    /// Maximum number of bytecode instructions before halting.
    /// Default: 50_000_000 (50M — enough for any reasonable program).
    pub max_instructions: u64,
    /// Maximum call stack depth.
    /// Default: 500 (prevents unbounded recursion).
    pub max_call_depth: usize,
    /// Maximum number of entries in any single list or dict value.
    /// Default: 1_000_000.
    pub max_collection_size: usize,
    /// Maximum byte length of any string value.
    /// Default: 10 * 1024 * 1024 (10 MB).
    pub max_string_bytes: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_instructions: 50_000_000,
            max_call_depth: 500,
            max_collection_size: 1_000_000,
            max_string_bytes: 10 * 1024 * 1024,
        }
    }
}

impl ExecutionLimits {
    /// Create limits suitable for untrusted / sandboxed code.
    pub fn sandboxed() -> Self {
        Self {
            max_instructions: 1_000_000,
            max_call_depth: 100,
            max_collection_size: 100_000,
            max_string_bytes: 1024 * 1024,
        }
    }

    /// Create limits suitable for development / REPL use (very generous).
    pub fn development() -> Self {
        Self {
            max_instructions: u64::MAX,
            max_call_depth: 10_000,
            max_collection_size: usize::MAX,
            max_string_bytes: usize::MAX,
        }
    }
}
