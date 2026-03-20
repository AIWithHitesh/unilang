// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Math built-in functions: abs, min, max, pow, sqrt, floor, ceil, round.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register math built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("abs", builtin_abs);
    vm.register_builtin("min", builtin_min);
    vm.register_builtin("max", builtin_max);
    vm.register_builtin("pow", builtin_pow);
    vm.register_builtin("sqrt", builtin_sqrt);
    vm.register_builtin("floor", builtin_floor);
    vm.register_builtin("ceil", builtin_ceil);
    vm.register_builtin("round", builtin_round);
}

fn builtin_abs(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("abs() requires 1 argument"))?;
    match val {
        RuntimeValue::Int(n) => Ok(RuntimeValue::Int(n.abs())),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Float(f.abs())),
        _ => Err(RuntimeError::type_error(format!(
            "abs() argument must be numeric, got {}",
            val
        ))),
    }
}

fn builtin_min(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "min() requires at least 2 arguments",
        ));
    }
    let a = &args[0];
    let b = &args[1];
    match a.partial_cmp(b) {
        Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => Ok(a.clone()),
        Some(std::cmp::Ordering::Greater) => Ok(b.clone()),
        None => Err(RuntimeError::type_error(format!(
            "cannot compare {} and {}",
            a, b
        ))),
    }
}

fn builtin_max(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "max() requires at least 2 arguments",
        ));
    }
    let a = &args[0];
    let b = &args[1];
    match a.partial_cmp(b) {
        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal) => Ok(a.clone()),
        Some(std::cmp::Ordering::Less) => Ok(b.clone()),
        None => Err(RuntimeError::type_error(format!(
            "cannot compare {} and {}",
            a, b
        ))),
    }
}

fn builtin_pow(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error("pow() requires 2 arguments"));
    }
    let base = &args[0];
    let exp = &args[1];
    match (base, exp) {
        (RuntimeValue::Int(b), RuntimeValue::Int(e)) => {
            if *e >= 0 {
                Ok(RuntimeValue::Int(b.pow(*e as u32)))
            } else {
                Ok(RuntimeValue::Float((*b as f64).powf(*e as f64)))
            }
        }
        _ => {
            let bf = base
                .as_float()
                .ok_or_else(|| RuntimeError::type_error("pow() requires numeric arguments"))?;
            let ef = exp
                .as_float()
                .ok_or_else(|| RuntimeError::type_error("pow() requires numeric arguments"))?;
            Ok(RuntimeValue::Float(bf.powf(ef)))
        }
    }
}

fn builtin_sqrt(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("sqrt() requires 1 argument"))?;
    let f = val
        .as_float()
        .ok_or_else(|| RuntimeError::type_error("sqrt() requires a numeric argument"))?;
    if f < 0.0 {
        return Err(RuntimeError::type_error("sqrt() of negative number"));
    }
    Ok(RuntimeValue::Float(f.sqrt()))
}

fn builtin_floor(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("floor() requires 1 argument"))?;
    match val {
        RuntimeValue::Int(n) => Ok(RuntimeValue::Int(*n)),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Int(f.floor() as i64)),
        _ => Err(RuntimeError::type_error(
            "floor() requires a numeric argument",
        )),
    }
}

fn builtin_ceil(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("ceil() requires 1 argument"))?;
    match val {
        RuntimeValue::Int(n) => Ok(RuntimeValue::Int(*n)),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Int(f.ceil() as i64)),
        _ => Err(RuntimeError::type_error(
            "ceil() requires a numeric argument",
        )),
    }
}

fn builtin_round(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("round() requires 1 argument"))?;
    match val {
        RuntimeValue::Int(n) => Ok(RuntimeValue::Int(*n)),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Int(f.round() as i64)),
        _ => Err(RuntimeError::type_error(
            "round() requires a numeric argument",
        )),
    }
}
