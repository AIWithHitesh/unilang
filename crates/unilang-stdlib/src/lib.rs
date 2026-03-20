// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang standard library — built-in functions and types
//! that are pre-registered in the runtime VM.

pub mod builtins;
pub mod collections;
pub mod math;
pub mod strings;

use unilang_runtime::vm::VM;

/// Register all built-in functions and values in the VM.
pub fn register_builtins(vm: &mut VM) {
    builtins::register_all(vm);
    math::register_all(vm);
    collections::register_all(vm);
    strings::register_all(vm);
}

#[cfg(test)]
mod tests;
