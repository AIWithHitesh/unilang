// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! String built-in functions: upper, lower, split, join, strip, replace,
//! contains, startswith, endswith.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register string built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("upper", builtin_upper);
    vm.register_builtin("lower", builtin_lower);
    vm.register_builtin("split", builtin_split);
    vm.register_builtin("join", builtin_join);
    vm.register_builtin("strip", builtin_strip);
    vm.register_builtin("replace", builtin_replace);
    vm.register_builtin("contains", builtin_contains);
    vm.register_builtin("startswith", builtin_startswith);
    vm.register_builtin("starts_with", builtin_startswith);
    vm.register_builtin("endswith", builtin_endswith);
    vm.register_builtin("ends_with", builtin_endswith);
}

fn expect_string<'a>(val: &'a RuntimeValue, func: &str) -> Result<&'a str, RuntimeError> {
    val.as_string()
        .ok_or_else(|| RuntimeError::type_error(format!("{}() requires a string argument", func)))
}

fn builtin_upper(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = expect_string(
        args.first()
            .ok_or_else(|| RuntimeError::type_error("upper() requires 1 argument"))?,
        "upper",
    )?;
    Ok(RuntimeValue::String(s.to_uppercase()))
}

fn builtin_lower(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = expect_string(
        args.first()
            .ok_or_else(|| RuntimeError::type_error("lower() requires 1 argument"))?,
        "lower",
    )?;
    Ok(RuntimeValue::String(s.to_lowercase()))
}

fn builtin_split(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::type_error(
            "split() requires at least 1 argument",
        ));
    }
    let s = expect_string(&args[0], "split")?;
    let sep = if args.len() > 1 {
        expect_string(&args[1], "split")?
    } else {
        " "
    };
    let parts: Vec<RuntimeValue> = s
        .split(sep)
        .map(|p| RuntimeValue::String(p.to_string()))
        .collect();
    Ok(RuntimeValue::List(parts))
}

fn builtin_join(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "join() requires 2 arguments: join(list, sep) or join(sep, list)",
        ));
    }
    // Accept both orderings: join(list, sep) or join(sep, list)
    let (list, sep): (&Vec<RuntimeValue>, &str) = match (&args[0], &args[1]) {
        (RuntimeValue::List(items), RuntimeValue::String(s)) => (items, s.as_str()),
        (RuntimeValue::String(s), RuntimeValue::List(items)) => (items, s.as_str()),
        _ => return Err(RuntimeError::type_error(
            "join() requires a list and a separator string",
        )),
    };
    let result: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
    Ok(RuntimeValue::String(result.join(sep)))
}

fn builtin_strip(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = expect_string(
        args.first()
            .ok_or_else(|| RuntimeError::type_error("strip() requires 1 argument"))?,
        "strip",
    )?;
    Ok(RuntimeValue::String(s.trim().to_string()))
}

fn builtin_replace(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 3 {
        return Err(RuntimeError::type_error(
            "replace() requires 3 arguments (string, old, new)",
        ));
    }
    let s = expect_string(&args[0], "replace")?;
    let old = expect_string(&args[1], "replace")?;
    let new = expect_string(&args[2], "replace")?;
    Ok(RuntimeValue::String(s.replace(old, new)))
}

fn builtin_contains(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "contains() requires 2 arguments (string, substring)",
        ));
    }
    let s = expect_string(&args[0], "contains")?;
    let substr = expect_string(&args[1], "contains")?;
    Ok(RuntimeValue::Bool(s.contains(substr)))
}

fn builtin_startswith(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "startswith() requires 2 arguments (string, prefix)",
        ));
    }
    let s = expect_string(&args[0], "startswith")?;
    let prefix = expect_string(&args[1], "startswith")?;
    Ok(RuntimeValue::Bool(s.starts_with(prefix)))
}

fn builtin_endswith(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "endswith() requires 2 arguments (string, suffix)",
        ));
    }
    let s = expect_string(&args[0], "endswith")?;
    let suffix = expect_string(&args[1], "endswith")?;
    Ok(RuntimeValue::Bool(s.ends_with(suffix)))
}
