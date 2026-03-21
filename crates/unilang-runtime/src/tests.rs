// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

use unilang_codegen::bytecode::{Bytecode, Function, Opcode, Value};

use crate::error::ErrorKind;
use crate::execute_with_output;
use crate::value::RuntimeValue;
use crate::vm::VM;

/// Helper to run instructions and return the VM.
fn run_instructions(
    instructions: Vec<Opcode>,
) -> (VM, Result<RuntimeValue, crate::error::RuntimeError>) {
    let bytecode = Bytecode {
        instructions,
        functions: Vec::new(),
        classes: Vec::new(),
    };
    let mut vm = VM::new();
    let result = vm.run(&bytecode);
    (vm, result)
}

/// Helper to run instructions with functions.
fn run_with_functions(
    instructions: Vec<Opcode>,
    functions: Vec<Function>,
) -> (VM, Result<RuntimeValue, crate::error::RuntimeError>) {
    let bytecode = Bytecode {
        instructions,
        functions,
        classes: Vec::new(),
    };
    let mut vm = VM::new();
    let result = vm.run(&bytecode);
    (vm, result)
}

// ── Test 1: LoadConst ──────────────────────────────────

#[test]
fn test_load_const() {
    let (_, result) = run_instructions(vec![Opcode::LoadConst(Value::Int(42)), Opcode::Halt]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 42),
        other => panic!("expected Int(42), got {:?}", other),
    }
}

// ── Test 2: Add int + int ──────────────────────────────

#[test]
fn test_add_int() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(3)),
        Opcode::LoadConst(Value::Int(7)),
        Opcode::Add,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 10),
        other => panic!("expected Int(10), got {:?}", other),
    }
}

// ── Test 3: Add string + string (concat) ───────────────

#[test]
fn test_add_string_concat() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::String("hello".into())),
        Opcode::LoadConst(Value::String(" world".into())),
        Opcode::Add,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::String(s) => assert_eq!(s, "hello world"),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── Test 4: Sub, Mul, Div ──────────────────────────────

#[test]
fn test_sub_mul_div() {
    // 10 - 3 = 7
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(10)),
        Opcode::LoadConst(Value::Int(3)),
        Opcode::Sub,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Int(7)));

    // 4 * 5 = 20
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(4)),
        Opcode::LoadConst(Value::Int(5)),
        Opcode::Mul,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Int(20)));

    // 10.0 / 4.0 = 2.5
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Float(10.0)),
        Opcode::LoadConst(Value::Float(4.0)),
        Opcode::Div,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Float(f) => assert!((f - 2.5).abs() < 1e-10),
        other => panic!("expected Float(2.5), got {:?}", other),
    }
}

// ── Test 5: Comparisons (Eq, Lt, Gt) ──────────────────

#[test]
fn test_comparisons() {
    // 5 == 5 → true
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(5)),
        Opcode::LoadConst(Value::Int(5)),
        Opcode::Eq,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(true)));

    // 3 < 5 → true
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(3)),
        Opcode::LoadConst(Value::Int(5)),
        Opcode::Lt,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(true)));

    // 7 > 2 → true
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(7)),
        Opcode::LoadConst(Value::Int(2)),
        Opcode::Gt,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(true)));

    // 5 == 3 → false
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(5)),
        Opcode::LoadConst(Value::Int(3)),
        Opcode::Eq,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(false)));
}

// ── Test 6: JumpIfFalse (takes branch) ─────────────────

#[test]
fn test_jump_if_false_takes_branch() {
    // Push false, JumpIfFalse to instruction 3, push 10 (skipped), push 20
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Bool(false)), // 0
        Opcode::JumpIfFalse(3),                // 1 - jump to 3
        Opcode::LoadConst(Value::Int(10)),     // 2 - skipped
        Opcode::LoadConst(Value::Int(20)),     // 3 - lands here
        Opcode::Halt,                          // 4
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 20),
        other => panic!("expected Int(20), got {:?}", other),
    }
}

// ── Test 7: JumpIfFalse (falls through) ────────────────

#[test]
fn test_jump_if_false_falls_through() {
    // Push true, JumpIfFalse (does not jump), push 10, halt
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Bool(true)), // 0
        Opcode::JumpIfFalse(3),               // 1 - does not jump
        Opcode::LoadConst(Value::Int(10)),    // 2 - executed
        Opcode::Halt,                         // 3
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 10),
        other => panic!("expected Int(10), got {:?}", other),
    }
}

// ── Test 8: While loop (count to 5) ────────────────────

#[test]
fn test_while_loop_count_to_5() {
    // i = 0; while i < 5: i = i + 1
    // locals[0] = i
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(0)), // 0: push 0
        Opcode::StoreLocal(0),            // 1: i = 0
        // loop start (ip=2)
        Opcode::LoadLocal(0),             // 2: push i
        Opcode::LoadConst(Value::Int(5)), // 3: push 5
        Opcode::Lt,                       // 4: i < 5
        Opcode::JumpIfFalse(11),          // 5: if false, jump to halt
        // body: i = i + 1
        Opcode::LoadLocal(0),             // 6: push i
        Opcode::LoadConst(Value::Int(1)), // 7: push 1
        Opcode::Add,                      // 8: i + 1
        Opcode::StoreLocal(0),            // 9: i = i + 1
        Opcode::Jump(2),                  // 10: loop back
        // after loop
        Opcode::LoadLocal(0), // 11: push i (should be 5)
        Opcode::Halt,         // 12
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 5),
        other => panic!("expected Int(5), got {:?}", other),
    }
}

// ── Test 9: Function call and return ───────────────────

#[test]
fn test_function_call_and_return() {
    // Define a function: fn add(a, b) { return a + b }
    let add_fn = Function {
        name: "add".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        code: vec![
            Opcode::LoadLocal(0), // push a
            Opcode::LoadLocal(1), // push b
            Opcode::Add,          // a + b
            Opcode::Return,       // return result
        ],
        local_count: 2,
    };

    // Main: push args, push function, call, halt
    let (_, result) = run_with_functions(
        vec![
            Opcode::LoadConst(Value::Int(3)), // 0: arg 1
            Opcode::LoadConst(Value::Int(4)), // 1: arg 2
            Opcode::MakeFunction(0),          // 2: push fn ref
            Opcode::Call(2),                  // 3: call with 2 args
            Opcode::Halt,                     // 4
        ],
        vec![add_fn],
    );
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 7),
        other => panic!("expected Int(7), got {:?}", other),
    }
}

// ── Test 10: Print ─────────────────────────────────────

#[test]
fn test_print() {
    let (vm, _result) = run_instructions(vec![
        Opcode::LoadConst(Value::String("hello".into())),
        Opcode::Print,
        Opcode::Halt,
    ]);
    assert_eq!(vm.output(), &["hello"]);
}

// ── Test 11: Variable store and load (local) ───────────

#[test]
fn test_local_variable_store_load() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(99)),
        Opcode::StoreLocal(0),
        Opcode::LoadLocal(0),
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 99),
        other => panic!("expected Int(99), got {:?}", other),
    }
}

// ── Test 12: Global store and load ─────────────────────

#[test]
fn test_global_variable_store_load() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(42)),
        Opcode::StoreGlobal("x".into()),
        Opcode::LoadGlobal("x".into()),
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 42),
        other => panic!("expected Int(42), got {:?}", other),
    }
}

// ── Test 13: MakeList ──────────────────────────────────

#[test]
fn test_make_list() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(1)),
        Opcode::LoadConst(Value::Int(2)),
        Opcode::LoadConst(Value::Int(3)),
        Opcode::MakeList(3),
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], RuntimeValue::Int(1)));
            assert!(matches!(items[1], RuntimeValue::Int(2)));
            assert!(matches!(items[2], RuntimeValue::Int(3)));
        }
        other => panic!("expected List, got {:?}", other),
    }
}

// ── Test 14: Division by zero → error ──────────────────

#[test]
fn test_division_by_zero() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(10)),
        Opcode::LoadConst(Value::Int(0)),
        Opcode::Div,
        Opcode::Halt,
    ]);
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::DivisionByZero);
}

// ── Test 15: Halt ──────────────────────────────────────

#[test]
fn test_halt() {
    // Halt should stop execution; the value before Halt is the result.
    let (_, result) = run_instructions(vec![Opcode::LoadConst(Value::Int(100)), Opcode::Halt]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 100),
        other => panic!("expected Int(100), got {:?}", other),
    }
}

// ── Test 16: Dup ───────────────────────────────────────

#[test]
fn test_dup() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(5)),
        Opcode::Dup,
        Opcode::Add,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 10),
        other => panic!("expected Int(10), got {:?}", other),
    }
}

// ── Test 17: Logical Not ───────────────────────────────

#[test]
fn test_logical_not() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Bool(true)),
        Opcode::Not,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(false)));
}

// ── Test 18: Negation ──────────────────────────────────

#[test]
fn test_negation() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(7)),
        Opcode::Neg,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, -7),
        other => panic!("expected Int(-7), got {:?}", other),
    }
}

// ── Test 19: Bitwise operations ────────────────────────

#[test]
fn test_bitwise_and() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(0b1100)),
        Opcode::LoadConst(Value::Int(0b1010)),
        Opcode::BitAnd,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 0b1000),
        other => panic!("expected Int(8), got {:?}", other),
    }
}

// ── Test 20: MakeDict ──────────────────────────────────

#[test]
fn test_make_dict() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::String("a".into())),
        Opcode::LoadConst(Value::Int(1)),
        Opcode::LoadConst(Value::String("b".into())),
        Opcode::LoadConst(Value::Int(2)),
        Opcode::MakeDict(2),
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Dict(pairs) => {
            assert_eq!(pairs.len(), 2);
        }
        other => panic!("expected Dict, got {:?}", other),
    }
}

// ── Test 21: Undefined variable error ──────────────────

#[test]
fn test_undefined_variable() {
    let (_, result) =
        run_instructions(vec![Opcode::LoadGlobal("nonexistent".into()), Opcode::Halt]);
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::UndefinedVariable);
}

// ── Test 22: GetIndex on list ──────────────────────────

#[test]
fn test_get_index() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(10)),
        Opcode::LoadConst(Value::Int(20)),
        Opcode::LoadConst(Value::Int(30)),
        Opcode::MakeList(3),
        Opcode::LoadConst(Value::Int(1)),
        Opcode::GetIndex,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Int(n) => assert_eq!(n, 20),
        other => panic!("expected Int(20), got {:?}", other),
    }
}

// ── Test 23: Concat ────────────────────────────────────

#[test]
fn test_concat() {
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::String("foo".into())),
        Opcode::LoadConst(Value::String("bar".into())),
        Opcode::Concat,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::String(s) => assert_eq!(s, "foobar"),
        other => panic!("expected String(foobar), got {:?}", other),
    }
}

// ── Test: Runtime int + float coercion ─────────────────

#[test]
fn test_runtime_int_plus_float() {
    // 5 + 3.14 → 8.14
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(5)),
        Opcode::LoadConst(Value::Float(3.14)),
        Opcode::Add,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Float(f) => assert!((f - 8.14).abs() < 1e-10),
        other => panic!("expected Float(8.14), got {:?}", other),
    }
}

// ── Test: Runtime string + int coercion (auto concat) ──

#[test]
fn test_runtime_string_concat_int() {
    // "hello" + 42 → "hello42"
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::String("hello".into())),
        Opcode::LoadConst(Value::Int(42)),
        Opcode::Add,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::String(s) => assert_eq!(s, "hello42"),
        other => panic!("expected String(hello42), got {:?}", other),
    }
}

// ── Test: Runtime int + string coercion (reverse) ──────

#[test]
fn test_runtime_int_concat_string() {
    // 42 + "hello" → "42hello"
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(42)),
        Opcode::LoadConst(Value::String("hello".into())),
        Opcode::Add,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::String(s) => assert_eq!(s, "42hello"),
        other => panic!("expected String(42hello), got {:?}", other),
    }
}

// ── Test: Runtime float + int subtraction ──────────────

#[test]
fn test_runtime_float_minus_int() {
    // 10.5 - 3 → 7.5
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Float(10.5)),
        Opcode::LoadConst(Value::Int(3)),
        Opcode::Sub,
        Opcode::Halt,
    ]);
    match result.unwrap() {
        RuntimeValue::Float(f) => assert!((f - 7.5).abs() < 1e-10),
        other => panic!("expected Float(7.5), got {:?}", other),
    }
}

// ── Test: Runtime int vs float comparison ──────────────

#[test]
fn test_runtime_mixed_comparison() {
    // 5 < 5.5 → true
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Int(5)),
        Opcode::LoadConst(Value::Float(5.5)),
        Opcode::Lt,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(true)));

    // 5.0 == 5 → true
    let (_, result) = run_instructions(vec![
        Opcode::LoadConst(Value::Float(5.0)),
        Opcode::LoadConst(Value::Int(5)),
        Opcode::Eq,
        Opcode::Halt,
    ]);
    assert!(matches!(result.unwrap(), RuntimeValue::Bool(true)));
}

// ── Test: RuntimeValue coercion methods ────────────────

#[test]
fn test_coerce_to_int() {
    assert_eq!(RuntimeValue::Int(42).coerce_to_int().unwrap(), 42);
    assert_eq!(RuntimeValue::Float(3.14).coerce_to_int().unwrap(), 3);
    assert_eq!(RuntimeValue::Bool(true).coerce_to_int().unwrap(), 1);
    assert_eq!(RuntimeValue::Null.coerce_to_int().unwrap(), 0);
    assert_eq!(
        RuntimeValue::String("123".to_string())
            .coerce_to_int()
            .unwrap(),
        123
    );
    assert!(RuntimeValue::String("abc".to_string())
        .coerce_to_int()
        .is_err());
}

#[test]
fn test_coerce_to_float() {
    assert!((RuntimeValue::Int(5).coerce_to_float().unwrap() - 5.0).abs() < 1e-10);
    assert!((RuntimeValue::Float(3.14).coerce_to_float().unwrap() - 3.14).abs() < 1e-10);
    assert!((RuntimeValue::Bool(true).coerce_to_float().unwrap() - 1.0).abs() < 1e-10);
    assert!((RuntimeValue::Null.coerce_to_float().unwrap() - 0.0).abs() < 1e-10);
}

#[test]
fn test_coerce_to_string() {
    assert_eq!(RuntimeValue::Int(42).coerce_to_string(), "42");
    assert_eq!(RuntimeValue::Float(3.14).coerce_to_string(), "3.14");
    assert_eq!(RuntimeValue::Bool(true).coerce_to_string(), "True");
    assert_eq!(RuntimeValue::Null.coerce_to_string(), "None");
    assert_eq!(
        RuntimeValue::String("hello".to_string()).coerce_to_string(),
        "hello"
    );
}

#[test]
fn test_coerce_to_bool() {
    assert!(RuntimeValue::Int(1).coerce_to_bool());
    assert!(!RuntimeValue::Int(0).coerce_to_bool());
    assert!(RuntimeValue::String("hello".to_string()).coerce_to_bool());
    assert!(!RuntimeValue::String("".to_string()).coerce_to_bool());
    assert!(!RuntimeValue::Null.coerce_to_bool());
}

// ── Integration test: full pipeline ────────────────────

#[test]
fn test_full_pipeline() {
    use unilang_common::span::SourceId;

    // Source: "x = 5 + 3\nprint(x)"
    let source = "x = 5 + 3\nprint(x)";

    // Lex and parse.
    let (module, _diag) = unilang_parser::parse(SourceId(0), source);

    // Compile to bytecode.
    let bytecode = unilang_codegen::compile(&module).expect("compilation failed");

    // Execute.
    let (result, output) = execute_with_output(&bytecode).expect("execution failed");

    // The print should have output "8".
    assert_eq!(output, vec!["8"]);

    // Result should be Null (print returns Null, then Halt).
    assert!(matches!(result, RuntimeValue::Null));
}
