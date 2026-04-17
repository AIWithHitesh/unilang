// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! End-to-end integration tests: parse → semantic → compile → execute.

use unilang_common::span::SourceId;
use unilang_runtime::value::RuntimeValue;

// ── Test helpers ──────────────────────────────────────────────────────────

/// Run a UniLang program source string through the full pipeline.
/// Returns (final RuntimeValue, captured output lines).
fn run_pipeline(source: &str) -> Result<(RuntimeValue, Vec<String>), String> {
    let sid = SourceId(0);
    let (module, parse_diag) = unilang_parser::parse(sid, source);
    if parse_diag.has_errors() {
        return Err(format!(
            "parse errors: {}",
            parse_diag
                .diagnostics()
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    let bytecode = unilang_codegen::compile(&module)
        .map_err(|diags| format!("compile errors: {} diag(s)", diags.len()))?;
    let mut vm = unilang_runtime::vm::VM::new_with_capture();
    unilang_stdlib::register_builtins(&mut vm);
    let result = vm
        .run(&bytecode)
        .map_err(|e| format!("runtime error: {}", e.message))?;
    let output = vm.output().to_vec();
    Ok((result, output))
}

/// Run and expect success; panic otherwise.
fn run_ok(source: &str) -> Vec<String> {
    match run_pipeline(source) {
        Ok((_, out)) => out,
        Err(e) => panic!("pipeline failed: {}", e),
    }
}

// ── 1. Hello World ────────────────────────────────────────────────────────

#[test]
fn test_hello_world() {
    let out = run_ok(r#"print("Hello, World!")"#);
    assert_eq!(out, vec!["Hello, World!"]);
}

// ── 2. Fibonacci ──────────────────────────────────────────────────────────

#[test]
fn test_fibonacci() {
    let source = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
print(fib(10))
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["55"]);
}

// ── 3. Class instantiation ────────────────────────────────────────────────

#[test]
fn test_class_instantiation() {
    // A class without __init__ can be instantiated without error.
    let source = r#"
class Vehicle:
    pass

v = Vehicle()
print("created")
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["created"]);
}

// ── 4. List operations ────────────────────────────────────────────────────

#[test]
fn test_list_operations() {
    let source = r#"
items = [3, 1, 4, 1, 5]
print(len(items))
s = sorted(items)
print(s[0])
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["5", "1"]);
}

// ── 5. Dict operations ────────────────────────────────────────────────────

#[test]
fn test_dict_operations() {
    let source = r#"
d = {"name": "Alice", "age": 30}
print(d["name"])
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["Alice"]);
}

// ── 6. Error handling (try/except) ────────────────────────────────────────

#[test]
fn test_error_handling() {
    let source = r#"
try:
    x = 1 / 0
except Exception as e:
    print("error caught")
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["error caught"]);
}

// ── 7. Nested for loops ───────────────────────────────────────────────────

#[test]
fn test_nested_for_loops() {
    let source = r#"
total = 0
for i in range(3):
    for j in range(3):
        total = total + 1
print(total)
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["9"]);
}

// ── 8. Higher-order function (function as value) ─────────────────────────

#[test]
fn test_higher_order_function() {
    let source = r#"
def apply(f, x):
    return f(x)

def double(n):
    return n * 2

print(apply(double, 21))
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["42"]);
}

// ── 9. String operations ──────────────────────────────────────────────────

#[test]
fn test_string_operations() {
    let source = r#"
s = "hello world"
print(len(s))
print(upper(s))
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["11", "HELLO WORLD"]);
}

// ── 10. Multiple return values via arithmetic ─────────────────────────────

#[test]
fn test_arithmetic_pipeline() {
    let source = r#"
def compute(a, b):
    sum_val = a + b
    product = a * b
    return sum_val + product

print(compute(3, 4))
"#;
    let out = run_ok(source);
    // sum = 7, product = 12, total = 19
    assert_eq!(out, vec!["19"]);
}
