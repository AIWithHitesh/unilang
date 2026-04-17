// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Criterion benchmarks for the UniLang bridge type-marshaling layer.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unilang_bridge::types::{bridge_to_runtime, runtime_to_bridge, BridgeValue};
use unilang_runtime::value::RuntimeValue;

fn bench_marshal_int(c: &mut Criterion) {
    let val = RuntimeValue::Int(42);
    c.bench_function("marshal_int: runtime→bridge", |b| {
        b.iter(|| runtime_to_bridge(black_box(&val)))
    });
}

fn bench_marshal_string(c: &mut Criterion) {
    let val = RuntimeValue::String("hello, world".to_string());
    c.bench_function("marshal_string: runtime→bridge", |b| {
        b.iter(|| runtime_to_bridge(black_box(&val)))
    });
}

fn bench_marshal_list(c: &mut Criterion) {
    let val = RuntimeValue::List(vec![
        RuntimeValue::Int(1),
        RuntimeValue::Float(2.0),
        RuntimeValue::String("three".to_string()),
        RuntimeValue::Bool(true),
        RuntimeValue::Null,
    ]);
    c.bench_function("marshal_list_5: runtime→bridge", |b| {
        b.iter(|| runtime_to_bridge(black_box(&val)))
    });
}

fn bench_marshal_dict(c: &mut Criterion) {
    let val = RuntimeValue::Dict(vec![
        (
            RuntimeValue::String("key1".to_string()),
            RuntimeValue::Int(100),
        ),
        (
            RuntimeValue::String("key2".to_string()),
            RuntimeValue::Float(3.14),
        ),
        (
            RuntimeValue::String("key3".to_string()),
            RuntimeValue::String("value".to_string()),
        ),
    ]);
    c.bench_function("marshal_dict_3: runtime→bridge", |b| {
        b.iter(|| runtime_to_bridge(black_box(&val)))
    });
}

fn bench_roundtrip_int(c: &mut Criterion) {
    let val = RuntimeValue::Int(12345);
    c.bench_function("roundtrip_int: runtime→bridge→runtime", |b| {
        b.iter(|| {
            let bridge = runtime_to_bridge(black_box(&val));
            bridge_to_runtime(black_box(bridge))
        })
    });
}

fn bench_roundtrip_string(c: &mut Criterion) {
    let val = RuntimeValue::String("benchmark string value".to_string());
    c.bench_function("roundtrip_string: runtime→bridge→runtime", |b| {
        b.iter(|| {
            let bridge = runtime_to_bridge(black_box(&val));
            bridge_to_runtime(black_box(bridge))
        })
    });
}

fn bench_bridge_to_runtime_java_object(c: &mut Criterion) {
    let val = BridgeValue::JavaObject {
        class: "java.lang.String".to_string(),
        handle: 42,
    };
    c.bench_function("bridge→runtime: JavaObject→String", |b| {
        b.iter(|| bridge_to_runtime(black_box(val.clone())))
    });
}

fn bench_bridge_to_runtime_python_object(c: &mut Criterion) {
    let val = BridgeValue::PythonObject {
        module: "numpy".to_string(),
        name: "ndarray".to_string(),
        handle: 7,
    };
    c.bench_function("bridge→runtime: PythonObject→String", |b| {
        b.iter(|| bridge_to_runtime(black_box(val.clone())))
    });
}

criterion_group!(
    benches,
    bench_marshal_int,
    bench_marshal_string,
    bench_marshal_list,
    bench_marshal_dict,
    bench_roundtrip_int,
    bench_roundtrip_string,
    bench_bridge_to_runtime_java_object,
    bench_bridge_to_runtime_python_object,
);
criterion_main!(benches);
