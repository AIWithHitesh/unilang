// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Math built-in functions: abs, min, max, pow, sqrt, floor, ceil, round,
//! log, log2, log10, sin, cos, tan, asin, acos, atan, atan2, exp, hypot,
//! gcd, factorial, clamp. Constants: PI, E.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register math built-in functions and global constants.
pub fn register_all(vm: &mut VM) {
    // Constants
    vm.set_global("PI", RuntimeValue::Float(std::f64::consts::PI));
    vm.set_global("E", RuntimeValue::Float(std::f64::consts::E));

    // Existing functions
    vm.register_builtin("abs", builtin_abs);
    vm.register_builtin("min", builtin_min);
    vm.register_builtin("max", builtin_max);
    vm.register_builtin("pow", builtin_pow);
    vm.register_builtin("sqrt", builtin_sqrt);
    vm.register_builtin("floor", builtin_floor);
    vm.register_builtin("ceil", builtin_ceil);
    vm.register_builtin("round", builtin_round);

    // Logarithms
    vm.register_builtin("log", builtin_log);
    vm.register_builtin("log2", builtin_log2);
    vm.register_builtin("log10", builtin_log10);

    // Trigonometry
    vm.register_builtin("sin", builtin_sin);
    vm.register_builtin("cos", builtin_cos);
    vm.register_builtin("tan", builtin_tan);
    vm.register_builtin("asin", builtin_asin);
    vm.register_builtin("acos", builtin_acos);
    vm.register_builtin("atan", builtin_atan);
    vm.register_builtin("atan2", builtin_atan2);

    // Exponential and geometric
    vm.register_builtin("exp", builtin_exp);
    vm.register_builtin("hypot", builtin_hypot);

    // Integer math
    vm.register_builtin("gcd", builtin_gcd);
    vm.register_builtin("factorial", builtin_factorial);

    // Range
    vm.register_builtin("clamp", builtin_clamp);
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
            other => {
                return Err(RuntimeError::type_error(format!(
                    "min() argument must be a list or multiple values, got {}",
                    other
                )))
            }
        }
    } else if args.len() >= 2 {
        args.iter().collect()
    } else {
        return Err(RuntimeError::type_error(
            "min() requires at least 1 argument",
        ));
    };
    let mut best = items[0];
    for item in &items[1..] {
        match best.partial_cmp(item) {
            Some(std::cmp::Ordering::Greater) => best = item,
            None => {
                return Err(RuntimeError::type_error(format!(
                    "cannot compare {} and {}",
                    best, item
                )))
            }
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
            other => {
                return Err(RuntimeError::type_error(format!(
                    "max() argument must be a list or multiple values, got {}",
                    other
                )))
            }
        }
    } else if args.len() >= 2 {
        args.iter().collect()
    } else {
        return Err(RuntimeError::type_error(
            "max() requires at least 1 argument",
        ));
    };
    let mut best = items[0];
    for item in &items[1..] {
        match best.partial_cmp(item) {
            Some(std::cmp::Ordering::Less) => best = item,
            None => {
                return Err(RuntimeError::type_error(format!(
                    "cannot compare {} and {}",
                    best, item
                )))
            }
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

// ── Helper: extract f64 from Int or Float ─────────────────────────────────────

fn numeric_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<f64, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n)) => Ok(*n as f64),
        Some(RuntimeValue::Float(f)) => Ok(*f),
        _ => Err(RuntimeError::type_error(format!(
            "{} requires a numeric argument at position {}",
            sig, idx
        ))),
    }
}

// ── Logarithms ────────────────────────────────────────────────────────────────

/// log(x)  → natural log;  log(x, base) → log_base(x)
fn builtin_log(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "log(x)")?;
    if x <= 0.0 {
        return Err(RuntimeError::type_error("log() argument must be positive"));
    }
    let result = if args.len() >= 2 {
        let base = numeric_arg(args, 1, "log(x, base)")?;
        if base <= 0.0 || base == 1.0 {
            return Err(RuntimeError::type_error(
                "log() base must be positive and not 1",
            ));
        }
        x.log(base)
    } else {
        x.ln()
    };
    Ok(RuntimeValue::Float(result))
}

fn builtin_log2(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "log2(x)")?;
    if x <= 0.0 {
        return Err(RuntimeError::type_error("log2() argument must be positive"));
    }
    Ok(RuntimeValue::Float(x.log2()))
}

fn builtin_log10(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "log10(x)")?;
    if x <= 0.0 {
        return Err(RuntimeError::type_error(
            "log10() argument must be positive",
        ));
    }
    Ok(RuntimeValue::Float(x.log10()))
}

// ── Trigonometry ──────────────────────────────────────────────────────────────

fn builtin_sin(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "sin(x)")?;
    Ok(RuntimeValue::Float(x.sin()))
}

fn builtin_cos(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "cos(x)")?;
    Ok(RuntimeValue::Float(x.cos()))
}

fn builtin_tan(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "tan(x)")?;
    Ok(RuntimeValue::Float(x.tan()))
}

fn builtin_asin(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "asin(x)")?;
    if !(-1.0..=1.0).contains(&x) {
        return Err(RuntimeError::type_error(
            "asin() argument must be in [-1, 1]",
        ));
    }
    Ok(RuntimeValue::Float(x.asin()))
}

fn builtin_acos(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "acos(x)")?;
    if !(-1.0..=1.0).contains(&x) {
        return Err(RuntimeError::type_error(
            "acos() argument must be in [-1, 1]",
        ));
    }
    Ok(RuntimeValue::Float(x.acos()))
}

fn builtin_atan(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "atan(x)")?;
    Ok(RuntimeValue::Float(x.atan()))
}

fn builtin_atan2(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let y = numeric_arg(args, 0, "atan2(y, x)")?;
    let x = numeric_arg(args, 1, "atan2(y, x)")?;
    Ok(RuntimeValue::Float(y.atan2(x)))
}

// ── Exponential & geometric ───────────────────────────────────────────────────

fn builtin_exp(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let x = numeric_arg(args, 0, "exp(x)")?;
    Ok(RuntimeValue::Float(x.exp()))
}

fn builtin_hypot(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let a = numeric_arg(args, 0, "hypot(a, b)")?;
    let b = numeric_arg(args, 1, "hypot(a, b)")?;
    Ok(RuntimeValue::Float(a.hypot(b)))
}

// ── Integer math ──────────────────────────────────────────────────────────────

fn builtin_gcd(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let a = match args.first() {
        Some(RuntimeValue::Int(n)) => n.unsigned_abs(),
        _ => {
            return Err(RuntimeError::type_error(
                "gcd() requires two integer arguments",
            ))
        }
    };
    let b = match args.get(1) {
        Some(RuntimeValue::Int(n)) => n.unsigned_abs(),
        _ => {
            return Err(RuntimeError::type_error(
                "gcd() requires two integer arguments",
            ))
        }
    };
    fn gcd_inner(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }
    Ok(RuntimeValue::Int(gcd_inner(a, b) as i64))
}

fn builtin_factorial(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let n = match args.first() {
        Some(RuntimeValue::Int(n)) => *n,
        _ => {
            return Err(RuntimeError::type_error(
                "factorial() requires an integer argument",
            ))
        }
    };
    if n < 0 {
        return Err(RuntimeError::type_error(
            "factorial() argument must be non-negative",
        ));
    }
    if n > 20 {
        return Err(RuntimeError::type_error(
            "factorial() argument must be <= 20 to avoid overflow",
        ));
    }
    let result: i64 = (1..=n).product();
    Ok(RuntimeValue::Int(result))
}

// ── Clamp ─────────────────────────────────────────────────────────────────────

fn builtin_clamp(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 3 {
        return Err(RuntimeError::type_error(
            "clamp() requires 3 arguments: value, min, max",
        ));
    }
    match (&args[0], &args[1], &args[2]) {
        (RuntimeValue::Int(v), RuntimeValue::Int(lo), RuntimeValue::Int(hi)) => {
            Ok(RuntimeValue::Int((*v).max(*lo).min(*hi)))
        }
        _ => {
            let v = args[0]
                .as_float()
                .ok_or_else(|| RuntimeError::type_error("clamp() requires numeric arguments"))?;
            let lo = args[1]
                .as_float()
                .ok_or_else(|| RuntimeError::type_error("clamp() requires numeric arguments"))?;
            let hi = args[2]
                .as_float()
                .ok_or_else(|| RuntimeError::type_error("clamp() requires numeric arguments"))?;
            Ok(RuntimeValue::Float(v.max(lo).min(hi)))
        }
    }
}
