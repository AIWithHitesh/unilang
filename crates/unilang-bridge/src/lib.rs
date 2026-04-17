// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang-bridge` — JVM/CPython interop for UniLang v2.0.
//!
//! This crate provides the type marshaling layer and bridge implementations
//! that enable UniLang programs to call into Java (via JNI) and CPython
//! (via the C API / pyo3).
//!
//! # Feature flags
//!
//! | Flag       | Description                                                          |
//! |------------|----------------------------------------------------------------------|
//! | `jvm`      | Enable the JNI bridge (requires a JVM installation and the `jni` crate) |
//! | `cpython`  | Enable the CPython bridge (requires Python headers and the `pyo3` crate) |
//!
//! Neither flag is on by default — all public API returns an appropriate
//! `BridgeError` explaining that the feature is not available.

/// Cross-VM error types.
pub mod error;

/// Type marshaling between [`unilang_runtime::value::RuntimeValue`] and [`types::BridgeValue`].
pub mod types;

/// JVM interop via JNI.
pub mod jvm;

/// CPython interop via pyo3.
pub mod cpython;

/// Zero-copy array bridge: JVM DirectByteBuffer ↔ NumPy buffer protocol.
pub mod arrays;

/// Java-backed thread pool bridge.
pub mod thread_pool;

/// UniLang VM builtin registration for JVM and CPython bridge functions.
pub mod driver;
