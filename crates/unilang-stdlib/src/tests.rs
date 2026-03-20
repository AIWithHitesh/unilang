// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Tests for the UniLang standard library.

use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::register_builtins;

/// Helper: create a VM with builtins registered, call a builtin by name.
fn call_builtin(vm: &mut VM, name: &str, args: &[RuntimeValue]) -> RuntimeValue {
    // Use the VM's builtin table directly via a bytecode that calls the function.
    // For unit tests, we invoke the functions through a small bytecode program.
    use unilang_codegen::bytecode::{Bytecode, Opcode, Value};

    // Build bytecode: push args, load the builtin, call it.
    let mut instructions = Vec::new();
    for arg in args {
        let const_val = match arg {
            RuntimeValue::Int(n) => Value::Int(*n),
            RuntimeValue::Float(f) => Value::Float(*f),
            RuntimeValue::String(s) => Value::String(s.clone()),
            RuntimeValue::Bool(b) => Value::Bool(*b),
            RuntimeValue::Null => Value::Null,
            _ => Value::Null, // For lists/dicts we'd need MakeList etc.
        };
        instructions.push(Opcode::LoadConst(const_val));
    }
    instructions.push(Opcode::LoadGlobal(name.to_string()));
    instructions.push(Opcode::Call(args.len()));
    instructions.push(Opcode::Halt);

    let bytecode = Bytecode {
        instructions,
        functions: Vec::new(),
        classes: Vec::new(),
    };

    vm.run(&bytecode).unwrap()
}

fn make_vm() -> VM {
    let mut vm = VM::new();
    register_builtins(&mut vm);
    vm
}

// ── len() ──────────────────────────────────────────

#[test]
fn test_len_string() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "len", &[RuntimeValue::String("hello".to_string())]);
    assert_eq!(result, RuntimeValue::Int(5));
}

#[test]
fn test_len_list() {
    // For list, we need to set up via bytecode with MakeList
    use unilang_codegen::bytecode::{Bytecode, Opcode, Value};

    let mut vm = make_vm();
    let bytecode = Bytecode {
        instructions: vec![
            Opcode::LoadConst(Value::Int(1)),
            Opcode::LoadConst(Value::Int(2)),
            Opcode::LoadConst(Value::Int(3)),
            Opcode::MakeList(3),
            Opcode::LoadGlobal("len".to_string()),
            Opcode::Call(1),
            Opcode::Halt,
        ],
        functions: Vec::new(),
        classes: Vec::new(),
    };
    let result = vm.run(&bytecode).unwrap();
    assert_eq!(result, RuntimeValue::Int(3));
}

#[test]
fn test_len_dict() {
    use unilang_codegen::bytecode::{Bytecode, Opcode, Value};

    let mut vm = make_vm();
    let bytecode = Bytecode {
        instructions: vec![
            Opcode::LoadConst(Value::String("a".to_string())),
            Opcode::LoadConst(Value::Int(1)),
            Opcode::LoadConst(Value::String("b".to_string())),
            Opcode::LoadConst(Value::Int(2)),
            Opcode::MakeDict(2),
            Opcode::LoadGlobal("len".to_string()),
            Opcode::Call(1),
            Opcode::Halt,
        ],
        functions: Vec::new(),
        classes: Vec::new(),
    };
    let result = vm.run(&bytecode).unwrap();
    assert_eq!(result, RuntimeValue::Int(2));
}

// ── range() ────────────────────────────────────────

#[test]
fn test_range_single_arg() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "range", &[RuntimeValue::Int(5)]);
    assert_eq!(
        result,
        RuntimeValue::List(vec![
            RuntimeValue::Int(0),
            RuntimeValue::Int(1),
            RuntimeValue::Int(2),
            RuntimeValue::Int(3),
            RuntimeValue::Int(4),
        ])
    );
}

#[test]
fn test_range_two_args() {
    let mut vm = make_vm();
    let result = call_builtin(
        &mut vm,
        "range",
        &[RuntimeValue::Int(2), RuntimeValue::Int(5)],
    );
    assert_eq!(
        result,
        RuntimeValue::List(vec![
            RuntimeValue::Int(2),
            RuntimeValue::Int(3),
            RuntimeValue::Int(4),
        ])
    );
}

// ── int/float/str conversions ──────────────────────

#[test]
fn test_int_from_float() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "int", &[RuntimeValue::Float(3.7)]);
    assert_eq!(result, RuntimeValue::Int(3));
}

#[test]
fn test_int_from_string() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "int", &[RuntimeValue::String("42".to_string())]);
    assert_eq!(result, RuntimeValue::Int(42));
}

#[test]
fn test_float_from_int() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "float", &[RuntimeValue::Int(5)]);
    assert_eq!(result, RuntimeValue::Float(5.0));
}

#[test]
fn test_str_from_int() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "str", &[RuntimeValue::Int(42)]);
    assert_eq!(result, RuntimeValue::String("42".to_string()));
}

// ── abs, min, max ──────────────────────────────────

#[test]
fn test_abs_negative() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "abs", &[RuntimeValue::Int(-7)]);
    assert_eq!(result, RuntimeValue::Int(7));
}

#[test]
fn test_abs_positive() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "abs", &[RuntimeValue::Int(5)]);
    assert_eq!(result, RuntimeValue::Int(5));
}

#[test]
fn test_min_ints() {
    let mut vm = make_vm();
    let result = call_builtin(
        &mut vm,
        "min",
        &[RuntimeValue::Int(3), RuntimeValue::Int(7)],
    );
    assert_eq!(result, RuntimeValue::Int(3));
}

#[test]
fn test_max_ints() {
    let mut vm = make_vm();
    let result = call_builtin(
        &mut vm,
        "max",
        &[RuntimeValue::Int(3), RuntimeValue::Int(7)],
    );
    assert_eq!(result, RuntimeValue::Int(7));
}

// ── sorted, reversed ───────────────────────────────

#[test]
fn test_sorted() {
    use unilang_codegen::bytecode::{Bytecode, Opcode, Value};

    let mut vm = make_vm();
    let bytecode = Bytecode {
        instructions: vec![
            Opcode::LoadConst(Value::Int(3)),
            Opcode::LoadConst(Value::Int(1)),
            Opcode::LoadConst(Value::Int(2)),
            Opcode::MakeList(3),
            Opcode::LoadGlobal("sorted".to_string()),
            Opcode::Call(1),
            Opcode::Halt,
        ],
        functions: Vec::new(),
        classes: Vec::new(),
    };
    let result = vm.run(&bytecode).unwrap();
    assert_eq!(
        result,
        RuntimeValue::List(vec![
            RuntimeValue::Int(1),
            RuntimeValue::Int(2),
            RuntimeValue::Int(3),
        ])
    );
}

#[test]
fn test_reversed() {
    use unilang_codegen::bytecode::{Bytecode, Opcode, Value};

    let mut vm = make_vm();
    let bytecode = Bytecode {
        instructions: vec![
            Opcode::LoadConst(Value::Int(1)),
            Opcode::LoadConst(Value::Int(2)),
            Opcode::LoadConst(Value::Int(3)),
            Opcode::MakeList(3),
            Opcode::LoadGlobal("reversed".to_string()),
            Opcode::Call(1),
            Opcode::Halt,
        ],
        functions: Vec::new(),
        classes: Vec::new(),
    };
    let result = vm.run(&bytecode).unwrap();
    assert_eq!(
        result,
        RuntimeValue::List(vec![
            RuntimeValue::Int(3),
            RuntimeValue::Int(2),
            RuntimeValue::Int(1),
        ])
    );
}

// ── upper, lower, split, join ──────────────────────

#[test]
fn test_upper() {
    let mut vm = make_vm();
    let result = call_builtin(
        &mut vm,
        "upper",
        &[RuntimeValue::String("hello".to_string())],
    );
    assert_eq!(result, RuntimeValue::String("HELLO".to_string()));
}

#[test]
fn test_lower() {
    let mut vm = make_vm();
    let result = call_builtin(
        &mut vm,
        "lower",
        &[RuntimeValue::String("HELLO".to_string())],
    );
    assert_eq!(result, RuntimeValue::String("hello".to_string()));
}

#[test]
fn test_split() {
    let mut vm = make_vm();
    let result = call_builtin(
        &mut vm,
        "split",
        &[
            RuntimeValue::String("a,b,c".to_string()),
            RuntimeValue::String(",".to_string()),
        ],
    );
    assert_eq!(
        result,
        RuntimeValue::List(vec![
            RuntimeValue::String("a".to_string()),
            RuntimeValue::String("b".to_string()),
            RuntimeValue::String("c".to_string()),
        ])
    );
}

#[test]
fn test_join() {
    use unilang_codegen::bytecode::{Bytecode, Opcode, Value};

    let mut vm = make_vm();
    let bytecode = Bytecode {
        instructions: vec![
            Opcode::LoadConst(Value::String(", ".to_string())),
            Opcode::LoadConst(Value::String("a".to_string())),
            Opcode::LoadConst(Value::String("b".to_string())),
            Opcode::LoadConst(Value::String("c".to_string())),
            Opcode::MakeList(3),
            Opcode::LoadGlobal("join".to_string()),
            Opcode::Call(2),
            Opcode::Halt,
        ],
        functions: Vec::new(),
        classes: Vec::new(),
    };
    let result = vm.run(&bytecode).unwrap();
    assert_eq!(result, RuntimeValue::String("a, b, c".to_string()));
}

// ── type() ─────────────────────────────────────────

#[test]
fn test_type_int() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "type", &[RuntimeValue::Int(42)]);
    assert_eq!(result, RuntimeValue::String("int".to_string()));
}

#[test]
fn test_type_string() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "type", &[RuntimeValue::String("hi".to_string())]);
    assert_eq!(result, RuntimeValue::String("str".to_string()));
}

#[test]
fn test_type_bool() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "type", &[RuntimeValue::Bool(true)]);
    assert_eq!(result, RuntimeValue::String("bool".to_string()));
}

#[test]
fn test_type_float() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "type", &[RuntimeValue::Float(3.14)]);
    assert_eq!(result, RuntimeValue::String("float".to_string()));
}

#[test]
fn test_type_null() {
    let mut vm = make_vm();
    let result = call_builtin(&mut vm, "type", &[RuntimeValue::Null]);
    assert_eq!(result, RuntimeValue::String("NoneType".to_string()));
}
