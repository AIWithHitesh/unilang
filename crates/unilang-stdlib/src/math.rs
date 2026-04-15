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
    // min(list) or min(a, b, ...)
    let items: Vec<&RuntimeValue> = if args.len() == 1 {
        match &args[0] {
            RuntimeValue::List(items) if !items.is_empty() => items.iter().collect(),
            RuntimeValue::List(_) => return Err(RuntimeError::type_error("min() of empty list")),
            other => return Err(RuntimeError::type_error(format!("min() argument must be a list or multiple values, got {}", other))),
        }
    } else if args.len() >= 2 {
        args.iter().collect()
    } else {
        return Err(RuntimeError::type_error("min() requires at least 1 argument"));
    };
    let mut best = items[0];
    for item in &items[1..] {
        match best.partial_cmp(item) {
            Some(std::cmp::Ordering::Greater) => best = item,
            None => return Err(RuntimeError::type_error(format!("cannot compare {} and {}", best, item))),
            _ => {}
        }
    }
    Ok(best.clone())
}

fn builtin_max(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    // max(list) or max(a, b, ...)
    let items: Vec<&RuntimeValue> = if args.len() == 1 {
        match &args[0] {
            RuntimeValue::List(items) if !items.is_empty() => items.iter().collect(),
            RuntimeValue::List(_) => return Err(RuntimeError::type_error("max() of empty list")),
            other => return Err(RuntimeError::type_error(format!("max() argument must be a list or multiple values, got {}", other))),
        }
    } else if args.len() >= 2 {
        args.iter().collect()
    } else {
        return Err(RuntimeError::type_error("max() requires at least 1 argument"));
    };
    let mut best = items[0];
    for item in &items[1..] {
        match best.partial_cmp(item) {
            Some(std::cmp::Ordering::Less) => best = item,
            None => return Err(RuntimeError::type_error(format!("cannot compare {} and {}", best, item))),
            _ => {}
        }
    }
    Ok(best.clone())
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
