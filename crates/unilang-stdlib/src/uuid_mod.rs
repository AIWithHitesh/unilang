// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UUID built-in functions: uuid_v4, uuid_is_valid, uuid_parse.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;
use uuid::Uuid;

/// Register UUID built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("uuid_v4", builtin_uuid_v4);
    vm.register_builtin("uuid_is_valid", builtin_uuid_is_valid);
    vm.register_builtin("uuid_parse", builtin_uuid_parse);
}

// ── Built-in functions ────────────────────────────────────────────────────────

fn builtin_uuid_v4(_args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    Ok(RuntimeValue::String(Uuid::new_v4().to_string()))
}

fn builtin_uuid_is_valid(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("uuid_is_valid() requires a string argument"))?;
    Ok(RuntimeValue::Bool(Uuid::parse_str(s).is_ok()))
}

fn builtin_uuid_parse(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("uuid_parse() requires a string argument"))?;
    Uuid::parse_str(s)
        .map(|u| RuntimeValue::String(u.hyphenated().to_string()))
        .map_err(|e| RuntimeError::type_error(format!("uuid_parse(): invalid UUID {:?}: {}", s, e)))
}
