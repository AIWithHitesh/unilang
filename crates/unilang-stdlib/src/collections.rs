// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Collection built-in functions: len, range, sorted, reversed, enumerate, zip.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register collection built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("len", builtin_len);
    vm.register_builtin("range", builtin_range);
    vm.register_builtin("sorted", builtin_sorted);
    vm.register_builtin("reversed", builtin_reversed);
    vm.register_builtin("enumerate", builtin_enumerate);
    vm.register_builtin("zip", builtin_zip);
}

fn builtin_len(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("len() requires 1 argument"))?;
    match val {
        RuntimeValue::String(s) => Ok(RuntimeValue::Int(s.len() as i64)),
        RuntimeValue::List(items) => Ok(RuntimeValue::Int(items.len() as i64)),
        RuntimeValue::Dict(pairs) => Ok(RuntimeValue::Int(pairs.len() as i64)),
        _ => Err(RuntimeError::type_error(format!(
            "object of type '{}' has no len()",
            type_name(val)
        ))),
    }
}

fn builtin_range(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (start, stop, step) = match args.len() {
        1 => {
            let stop = args[0]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("range() requires integer arguments"))?;
            (0i64, stop, 1i64)
        }
        2 => {
            let start = args[0]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("range() requires integer arguments"))?;
            let stop = args[1]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("range() requires integer arguments"))?;
            (start, stop, 1i64)
        }
        3 => {
            let start = args[0]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("range() requires integer arguments"))?;
            let stop = args[1]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("range() requires integer arguments"))?;
            let step = args[2]
                .as_int()
                .ok_or_else(|| RuntimeError::type_error("range() requires integer arguments"))?;
            if step == 0 {
                return Err(RuntimeError::type_error("range() step must not be zero"));
            }
            (start, stop, step)
        }
        _ => {
            return Err(RuntimeError::type_error(
                "range() requires 1, 2, or 3 arguments",
            ))
        }
    };

    let mut result = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < stop {
            result.push(RuntimeValue::Int(i));
            i += step;
        }
    } else {
        while i > stop {
            result.push(RuntimeValue::Int(i));
            i += step;
        }
    }
    Ok(RuntimeValue::List(result))
}

fn builtin_sorted(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("sorted() requires 1 argument"))?;
    match val {
        RuntimeValue::List(items) => {
            let mut sorted = items.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Ok(RuntimeValue::List(sorted))
        }
        _ => Err(RuntimeError::type_error("sorted() requires a list")),
    }
}

fn builtin_reversed(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("reversed() requires 1 argument"))?;
    match val {
        RuntimeValue::List(items) => {
            let mut rev = items.clone();
            rev.reverse();
            Ok(RuntimeValue::List(rev))
        }
        _ => Err(RuntimeError::type_error("reversed() requires a list")),
    }
}

fn builtin_enumerate(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("enumerate() requires 1 argument"))?;
    match val {
        RuntimeValue::List(items) => {
            let result: Vec<RuntimeValue> = items
                .iter()
                .enumerate()
                .map(|(i, v)| RuntimeValue::List(vec![RuntimeValue::Int(i as i64), v.clone()]))
                .collect();
            Ok(RuntimeValue::List(result))
        }
        _ => Err(RuntimeError::type_error("enumerate() requires a list")),
    }
}

fn builtin_zip(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "zip() requires at least 2 arguments",
        ));
    }
    let list1 = match &args[0] {
        RuntimeValue::List(items) => items,
        _ => return Err(RuntimeError::type_error("zip() requires lists")),
    };
    let list2 = match &args[1] {
        RuntimeValue::List(items) => items,
        _ => return Err(RuntimeError::type_error("zip() requires lists")),
    };
    let len = list1.len().min(list2.len());
    let result: Vec<RuntimeValue> = (0..len)
        .map(|i| RuntimeValue::List(vec![list1[i].clone(), list2[i].clone()]))
        .collect();
    Ok(RuntimeValue::List(result))
}

fn type_name(val: &RuntimeValue) -> &str {
    match val {
        RuntimeValue::Int(_) => "int",
        RuntimeValue::Float(_) => "float",
        RuntimeValue::String(_) => "str",
        RuntimeValue::Bool(_) => "bool",
        RuntimeValue::Null => "NoneType",
        RuntimeValue::List(_) => "list",
        RuntimeValue::Dict(_) => "dict",
        RuntimeValue::Function(_) => "function",
        RuntimeValue::Instance(data) => data.class_name.as_str(),
        RuntimeValue::NativeFunction(_) => "builtin_function",
    }
}
