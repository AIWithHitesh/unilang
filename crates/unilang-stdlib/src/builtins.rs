// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Core built-in functions: I/O, type conversion, type checking, utility.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register core built-in functions.
pub fn register_all(vm: &mut VM) {
    // I/O
    vm.register_builtin("print", builtin_print);
    vm.register_builtin("input", builtin_input);

    // Type conversion
    vm.register_builtin("int", builtin_int);
    vm.register_builtin("float", builtin_float);
    vm.register_builtin("str", builtin_str);
    vm.register_builtin("bool", builtin_bool);

    // Type checking
    vm.register_builtin("type", builtin_type);
    vm.register_builtin("isinstance", builtin_isinstance);

    // Utility
    vm.register_builtin("hash", builtin_hash);
}

fn builtin_print(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let text = args
        .iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", text);
    Ok(RuntimeValue::Null)
}

fn builtin_input(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if let Some(prompt) = args.first() {
        print!("{}", prompt);
    }
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| RuntimeError::type_error(format!("I/O error: {}", e)))?;
    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(RuntimeValue::String(line))
}

fn builtin_int(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("int() requires 1 argument"))?;
    match val {
        RuntimeValue::Int(n) => Ok(RuntimeValue::Int(*n)),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Int(*f as i64)),
        RuntimeValue::Bool(b) => Ok(RuntimeValue::Int(if *b { 1 } else { 0 })),
        RuntimeValue::String(s) => s
            .trim()
            .parse::<i64>()
            .map(RuntimeValue::Int)
            .map_err(|_| RuntimeError::type_error(format!("cannot convert '{}' to int", s))),
        _ => Err(RuntimeError::type_error(format!(
            "cannot convert {} to int",
            val
        ))),
    }
}

fn builtin_float(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("float() requires 1 argument"))?;
    match val {
        RuntimeValue::Float(f) => Ok(RuntimeValue::Float(*f)),
        RuntimeValue::Int(n) => Ok(RuntimeValue::Float(*n as f64)),
        RuntimeValue::Bool(b) => Ok(RuntimeValue::Float(if *b { 1.0 } else { 0.0 })),
        RuntimeValue::String(s) => s
            .trim()
            .parse::<f64>()
            .map(RuntimeValue::Float)
            .map_err(|_| RuntimeError::type_error(format!("cannot convert '{}' to float", s))),
        _ => Err(RuntimeError::type_error(format!(
            "cannot convert {} to float",
            val
        ))),
    }
}

fn builtin_str(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("str() requires 1 argument"))?;
    Ok(RuntimeValue::String(format!("{}", val)))
}

fn builtin_bool(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("bool() requires 1 argument"))?;
    Ok(RuntimeValue::Bool(val.is_truthy()))
}

fn builtin_type(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("type() requires 1 argument"))?;
    let name = match val {
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
    };
    Ok(RuntimeValue::String(name.to_string()))
}

fn builtin_isinstance(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "isinstance() requires 2 arguments",
        ));
    }
    let val = &args[0];
    let type_name = match &args[1] {
        RuntimeValue::String(s) => s.as_str(),
        _ => {
            return Err(RuntimeError::type_error(
                "isinstance() second argument must be a string type name",
            ))
        }
    };
    let actual = match val {
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
    };
    Ok(RuntimeValue::Bool(actual == type_name))
}

fn builtin_hash(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("hash() requires 1 argument"))?;
    let mut hasher = DefaultHasher::new();
    match val {
        RuntimeValue::Int(n) => n.hash(&mut hasher),
        RuntimeValue::String(s) => s.hash(&mut hasher),
        RuntimeValue::Bool(b) => b.hash(&mut hasher),
        RuntimeValue::Null => 0_i64.hash(&mut hasher),
        _ => {
            return Err(RuntimeError::type_error(format!(
                "unhashable type: {}",
                builtin_type(&[val.clone()])
                    .map(|v| format!("{}", v))
                    .unwrap_or_default()
            )))
        }
    }
    Ok(RuntimeValue::Int(hasher.finish() as i64))
}
