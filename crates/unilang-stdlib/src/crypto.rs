// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Crypto built-in functions: sha256, sha512, md5, hmac_sha256, hash_sha256.

use hmac::{Hmac, Mac};
use md5::Md5;
use sha2::{Digest, Sha256, Sha512};
use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

type HmacSha256 = Hmac<Sha256>;

/// Register Crypto built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("sha256", builtin_sha256);
    vm.register_builtin("sha512", builtin_sha512);
    vm.register_builtin("md5", builtin_md5);
    vm.register_builtin("hmac_sha256", builtin_hmac_sha256);
    vm.register_builtin("hash_sha256", builtin_sha256);
}

// ── Built-in functions ────────────────────────────────────────────────────────

fn builtin_sha256(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("sha256() requires a string argument"))?;
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(RuntimeValue::String(hex::encode(result)))
}

fn builtin_sha512(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("sha512() requires a string argument"))?;
    let mut hasher = Sha512::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(RuntimeValue::String(hex::encode(result)))
}

fn builtin_md5(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("md5() requires a string argument"))?;
    let mut hasher = Md5::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(RuntimeValue::String(hex::encode(result)))
}

fn builtin_hmac_sha256(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "hmac_sha256() requires two string arguments: key and message",
        ));
    }
    let key = args[0]
        .as_string()
        .ok_or_else(|| RuntimeError::type_error("hmac_sha256(): key must be a string"))?;
    let message = args[1]
        .as_string()
        .ok_or_else(|| RuntimeError::type_error("hmac_sha256(): message must be a string"))?;
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .map_err(|e| RuntimeError::type_error(format!("hmac_sha256(): invalid key: {}", e)))?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    Ok(RuntimeValue::String(hex::encode(result.into_bytes())))
}
