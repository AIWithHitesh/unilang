// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Core built-in functions: I/O, type conversion, type checking, utility.

use std::collections::HashMap;

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::{InstanceData, RuntimeValue};
use unilang_runtime::vm::VM;

/// Minimal `System.out.println` facade for Java-style hello-world code (not the real JVM).
fn java_lang_system() -> RuntimeValue {
    let mut out_fields = HashMap::new();
    out_fields.insert(
        "println".to_string(),
        RuntimeValue::NativeFunction("println".to_string()),
    );
    let out = RuntimeValue::Instance(InstanceData {
        class_name: "PrintStream".to_string(),
        fields: out_fields,
    });
    let mut sys_fields = HashMap::new();
    sys_fields.insert("out".to_string(), out);
    RuntimeValue::Instance(InstanceData {
        class_name: "java.lang.System".to_string(),
        fields: sys_fields,
    })
}

/// Register core built-in functions.
pub fn register_all(vm: &mut VM) {
    // I/O
    vm.register_builtin("print", builtin_print);
    vm.register_builtin("println", builtin_print);
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
    vm.register_builtin("id", builtin_id);

    // Aggregates
    vm.register_builtin("sum", builtin_sum);
    vm.register_builtin("any", builtin_any);
    vm.register_builtin("all", builtin_all);

    // Character conversion
    vm.register_builtin("chr", builtin_chr);
    vm.register_builtin("ord", builtin_ord);

    // Collection constructors
    vm.register_builtin("list", builtin_list);
    vm.register_builtin("dict", builtin_dict);

    // I/O extras
    vm.register_builtin("format", builtin_format);

    // File I/O
    vm.register_builtin("read_file", builtin_read_file);
    vm.register_builtin("file_exists", builtin_file_exists);

    vm.set_global("System", java_lang_system());
    vm.set_global("None", RuntimeValue::Null);
    vm.set_global("True", RuntimeValue::Bool(true));
    vm.set_global("False", RuntimeValue::Bool(false));

    // HTTP server — handled specially in VM::call_builtin (needs &mut self)
    vm.set_global("serve", RuntimeValue::NativeFunction("serve".to_string()));
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

fn builtin_id(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args.first().ok_or_else(|| RuntimeError::type_error("id() requires 1 argument"))?;
    // Return a stable integer representation.
    let id = match val {
        RuntimeValue::Int(n) => *n,
        RuntimeValue::Bool(b) => if *b { 1 } else { 0 },
        _ => 0,
    };
    Ok(RuntimeValue::Int(id))
}

fn builtin_sum(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args.first().ok_or_else(|| RuntimeError::type_error("sum() requires 1 argument"))?;
    match val {
        RuntimeValue::List(items) => {
            let mut total_int: i64 = 0;
            let mut total_float: f64 = 0.0;
            let mut has_float = false;
            for item in items {
                match item {
                    RuntimeValue::Int(n) => { total_int += n; total_float += *n as f64; }
                    RuntimeValue::Float(f) => { total_float += f; has_float = true; }
                    _ => return Err(RuntimeError::type_error(format!("sum() element '{}' is not numeric", item))),
                }
            }
            if has_float {
                Ok(RuntimeValue::Float(total_float))
            } else {
                Ok(RuntimeValue::Int(total_int))
            }
        }
        _ => Err(RuntimeError::type_error("sum() requires a list")),
    }
}

fn builtin_any(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args.first().ok_or_else(|| RuntimeError::type_error("any() requires 1 argument"))?;
    match val {
        RuntimeValue::List(items) => Ok(RuntimeValue::Bool(items.iter().any(|v| v.is_truthy()))),
        _ => Err(RuntimeError::type_error("any() requires a list")),
    }
}

fn builtin_all(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args.first().ok_or_else(|| RuntimeError::type_error("all() requires 1 argument"))?;
    match val {
        RuntimeValue::List(items) => Ok(RuntimeValue::Bool(items.iter().all(|v| v.is_truthy()))),
        _ => Err(RuntimeError::type_error("all() requires a list")),
    }
}

fn builtin_chr(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let n = args.first().and_then(|v| v.as_int())
        .ok_or_else(|| RuntimeError::type_error("chr() requires an integer argument"))?;
    let c = char::from_u32(n as u32)
        .ok_or_else(|| RuntimeError::type_error(format!("chr() argument {} is not a valid Unicode code point", n)))?;
    Ok(RuntimeValue::String(c.to_string()))
}

fn builtin_ord(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args.first().and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("ord() requires a string argument"))?;
    let mut chars = s.chars();
    let c = chars.next().ok_or_else(|| RuntimeError::type_error("ord() argument is an empty string"))?;
    if chars.next().is_some() {
        return Err(RuntimeError::type_error("ord() expects a single character, not a multi-character string"));
    }
    Ok(RuntimeValue::Int(c as i64))
}

fn builtin_list(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    match args.first() {
        None => Ok(RuntimeValue::List(Vec::new())),
        Some(RuntimeValue::List(items)) => Ok(RuntimeValue::List(items.clone())),
        Some(RuntimeValue::String(s)) => {
            Ok(RuntimeValue::List(s.chars().map(|c| RuntimeValue::String(c.to_string())).collect()))
        }
        Some(RuntimeValue::Dict(pairs)) => {
            Ok(RuntimeValue::List(pairs.iter().map(|(k, _)| k.clone()).collect()))
        }
        Some(other) => Err(RuntimeError::type_error(format!("list() cannot convert '{}'", other))),
    }
}

fn builtin_dict(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    match args.first() {
        None => Ok(RuntimeValue::Dict(Vec::new())),
        Some(RuntimeValue::Dict(pairs)) => Ok(RuntimeValue::Dict(pairs.clone())),
        Some(RuntimeValue::List(items)) => {
            // list of [key, value] pairs
            let mut pairs = Vec::new();
            for item in items {
                match item {
                    RuntimeValue::List(pair) if pair.len() >= 2 => {
                        pairs.push((pair[0].clone(), pair[1].clone()));
                    }
                    _ => return Err(RuntimeError::type_error("dict() list must contain [key, value] pairs")),
                }
            }
            Ok(RuntimeValue::Dict(pairs))
        }
        Some(other) => Err(RuntimeError::type_error(format!("dict() cannot convert '{}'", other))),
    }
}

fn builtin_format(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::type_error("format() requires at least 1 argument"));
    }
    // Simple positional format: format("{} + {} = {}", a, b, c)
    let template = args[0].as_string()
        .ok_or_else(|| RuntimeError::type_error("format() first argument must be a string"))?
        .to_string();
    let mut result = String::new();
    let mut arg_idx = 1;
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'}') {
                chars.next();
                if arg_idx < args.len() {
                    result.push_str(&format!("{}", args[arg_idx]));
                    arg_idx += 1;
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    Ok(RuntimeValue::String(result))
}

fn builtin_read_file(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let path = match args.first() {
        Some(RuntimeValue::String(s)) => s.clone(),
        _ => return Err(RuntimeError::type_error("read_file() requires a string path")),
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(RuntimeValue::String(content)),
        Err(e) => Err(RuntimeError::type_error(format!(
            "read_file('{}') failed: {}",
            path, e
        ))),
    }
}

fn builtin_file_exists(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let path = match args.first() {
        Some(RuntimeValue::String(s)) => s.clone(),
        _ => return Err(RuntimeError::type_error("file_exists() requires a string path")),
    };
    Ok(RuntimeValue::Bool(std::path::Path::new(&path).exists()))
}
