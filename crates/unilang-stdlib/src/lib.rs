// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang standard library — built-in functions and types
//! that are pre-registered in the runtime VM.

pub mod base64_mod;
pub mod builtins;
pub mod collections;
pub mod crypto;
pub mod csv_mod;
pub mod datetime;
pub mod json;
pub mod math;
pub mod regex_mod;
pub mod strings;
pub mod uuid_mod;

use unilang_runtime::vm::VM;

/// Register all built-in functions and values in the VM.
pub fn register_builtins(vm: &mut VM) {
    builtins::register_all(vm);
    json::register_all(vm);
    math::register_all(vm);
    collections::register_all(vm);
    strings::register_all(vm);
    datetime::register_all(vm);
    regex_mod::register_all(vm);
    uuid_mod::register_all(vm);
    base64_mod::register_all(vm);
    crypto::register_all(vm);
    csv_mod::register_all(vm);
}

#[cfg(test)]
mod tests;
