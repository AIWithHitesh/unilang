// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Regex built-in functions: regex_match, regex_match_full, regex_find,
//! regex_find_all, regex_replace, regex_replace_all, regex_split, regex_groups.

use regex::Regex;
use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register regex built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("regex_match", builtin_regex_match);
    vm.register_builtin("regex_match_full", builtin_regex_match_full);
    vm.register_builtin("regex_find", builtin_regex_find);
    vm.register_builtin("regex_find_all", builtin_regex_find_all);
    vm.register_builtin("regex_replace", builtin_regex_replace);
    vm.register_builtin("regex_replace_all", builtin_regex_replace_all);
    vm.register_builtin("regex_split", builtin_regex_split);
    vm.register_builtin("regex_groups", builtin_regex_groups);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn compile_regex(pattern: &str) -> Result<Regex, RuntimeError> {
    Regex::new(pattern).map_err(|e| {
        RuntimeError::type_error(format!(
            "regex_compile(): invalid pattern {:?}: {}",
            pattern, e
        ))
    })
}

fn get_pattern_and_text(
    args: &[RuntimeValue],
    func: &str,
) -> Result<(String, String), RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(format!(
            "{}() requires at least 2 arguments: {}(pattern, text)",
            func, func
        )));
    }
    let pattern = args[0]
        .as_string()
        .ok_or_else(|| RuntimeError::type_error(format!("{}() pattern must be a string", func)))?
        .to_string();
    let text = args[1]
        .as_string()
        .ok_or_else(|| RuntimeError::type_error(format!("{}() text must be a string", func)))?
        .to_string();
    Ok((pattern, text))
}

// ── Built-in functions ────────────────────────────────────────────────────────

fn builtin_regex_match(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (pattern, text) = get_pattern_and_text(args, "regex_match")?;
    let re = compile_regex(&pattern)?;
    Ok(RuntimeValue::Bool(re.is_match(&text)))
}

fn builtin_regex_match_full(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (pattern, text) = get_pattern_and_text(args, "regex_match_full")?;
    // Anchor the pattern to require a full match
    let anchored = format!("^(?:{})$", pattern);
    let re = compile_regex(&anchored)?;
    Ok(RuntimeValue::Bool(re.is_match(&text)))
}

fn builtin_regex_find(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (pattern, text) = get_pattern_and_text(args, "regex_find")?;
    let re = compile_regex(&pattern)?;
    match re.find(&text) {
        Some(m) => Ok(RuntimeValue::String(m.as_str().to_string())),
        None => Ok(RuntimeValue::Null),
    }
}

fn builtin_regex_find_all(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (pattern, text) = get_pattern_and_text(args, "regex_find_all")?;
    let re = compile_regex(&pattern)?;
    let matches: Vec<RuntimeValue> = re
        .find_iter(&text)
        .map(|m| RuntimeValue::String(m.as_str().to_string()))
        .collect();
    Ok(RuntimeValue::List(matches))
}

fn builtin_regex_replace(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 3 {
        return Err(RuntimeError::type_error(
            "regex_replace() requires 3 arguments: regex_replace(pattern, text, replacement)",
        ));
    }
    let (pattern, text) = get_pattern_and_text(args, "regex_replace")?;
    let replacement = args[2]
        .as_string()
        .ok_or_else(|| RuntimeError::type_error("regex_replace() replacement must be a string"))?;
    let re = compile_regex(&pattern)?;
    Ok(RuntimeValue::String(
        re.replace(&text, replacement).into_owned(),
    ))
}

fn builtin_regex_replace_all(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 3 {
        return Err(RuntimeError::type_error(
            "regex_replace_all() requires 3 arguments: regex_replace_all(pattern, text, replacement)",
        ));
    }
    let (pattern, text) = get_pattern_and_text(args, "regex_replace_all")?;
    let replacement = args[2].as_string().ok_or_else(|| {
        RuntimeError::type_error("regex_replace_all() replacement must be a string")
    })?;
    let re = compile_regex(&pattern)?;
    Ok(RuntimeValue::String(
        re.replace_all(&text, replacement).into_owned(),
    ))
}

fn builtin_regex_split(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (pattern, text) = get_pattern_and_text(args, "regex_split")?;
    let re = compile_regex(&pattern)?;
    let parts: Vec<RuntimeValue> = re
        .split(&text)
        .map(|s| RuntimeValue::String(s.to_string()))
        .collect();
    Ok(RuntimeValue::List(parts))
}

fn builtin_regex_groups(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let (pattern, text) = get_pattern_and_text(args, "regex_groups")?;
    let re = compile_regex(&pattern)?;
    match re.captures(&text) {
        Some(caps) => {
            // Skip the full match (index 0), return only capture groups
            let groups: Vec<RuntimeValue> = caps
                .iter()
                .skip(1)
                .map(|m| match m {
                    Some(matched) => RuntimeValue::String(matched.as_str().to_string()),
                    None => RuntimeValue::Null,
                })
                .collect();
            Ok(RuntimeValue::List(groups))
        }
        None => Ok(RuntimeValue::Null),
    }
}
