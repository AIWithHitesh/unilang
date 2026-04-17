// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Base64 built-in functions: base64_encode, base64_decode,
//! base64_encode_url, base64_decode_url.

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register Base64 built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("base64_encode", builtin_base64_encode);
    vm.register_builtin("base64_decode", builtin_base64_decode);
    vm.register_builtin("base64_encode_url", builtin_base64_encode_url);
    vm.register_builtin("base64_decode_url", builtin_base64_decode_url);
}

// ── Built-in functions ────────────────────────────────────────────────────────

fn builtin_base64_encode(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("base64_encode() requires a string argument"))?;
    Ok(RuntimeValue::String(STANDARD.encode(s.as_bytes())))
}

fn builtin_base64_decode(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("base64_decode() requires a string argument"))?;
    let bytes = STANDARD.decode(s).map_err(|e| {
        RuntimeError::type_error(format!("base64_decode(): invalid base64 input: {}", e))
    })?;
    String::from_utf8(bytes)
        .map(RuntimeValue::String)
        .map_err(|e| {
            RuntimeError::type_error(format!(
                "base64_decode(): decoded bytes are not valid UTF-8: {}",
                e
            ))
        })
}

fn builtin_base64_encode_url(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args.first().and_then(|v| v.as_string()).ok_or_else(|| {
        RuntimeError::type_error("base64_encode_url() requires a string argument")
    })?;
    Ok(RuntimeValue::String(URL_SAFE_NO_PAD.encode(s.as_bytes())))
}

fn builtin_base64_decode_url(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args.first().and_then(|v| v.as_string()).ok_or_else(|| {
        RuntimeError::type_error("base64_decode_url() requires a string argument")
    })?;
    let bytes = URL_SAFE_NO_PAD.decode(s).map_err(|e| {
        RuntimeError::type_error(format!(
            "base64_decode_url(): invalid URL-safe base64 input: {}",
            e
        ))
    })?;
    String::from_utf8(bytes)
        .map(RuntimeValue::String)
        .map_err(|e| {
            RuntimeError::type_error(format!(
                "base64_decode_url(): decoded bytes are not valid UTF-8: {}",
                e
            ))
        })
}
