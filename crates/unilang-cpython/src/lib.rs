// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # unilang-cpython
//!
//! A bridge that lets UniLang programs call CPython libraries (numpy, sklearn,
//! pandas, etc.) via PyO3.
//!
//! ## Features
//!
//! | Cargo feature | Behaviour |
//! |---|---|
//! | *(none — default)* | All bridge functions return a stub error describing how to enable CPython. |
//! | `cpython` | Full PyO3-backed implementation; requires Python 3.x dev headers. |
//!
//! ## UniLang builtins registered
//!
//! | Name | Signature | Description |
//! |---|---|---|
//! | `py_import` | `(module: String) -> Bool` | Import a Python module into the bridge state. |
//! | `py_call` | `(module: String, func: String, ...args) -> RuntimeValue` | Call a function from a previously imported module. |
//! | `py_eval` | `(expr: String) -> RuntimeValue` | Evaluate a Python expression and return the result. |
//!
//! ## Example (UniLang source)
//!
//! ```text
//! py_import("math")
//! result = py_call("math", "sqrt", 144.0)
//! print(result)   # => 12.0
//! ```

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

// ── Bridge error type ─────────────────────────────────────────────────────────

/// An error produced by the CPython bridge.
#[derive(Debug, Clone)]
pub struct BridgeError(pub String);

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cpython bridge error: {}", self.0)
    }
}

impl std::error::Error for BridgeError {}

impl From<BridgeError> for RuntimeError {
    fn from(e: BridgeError) -> Self {
        RuntimeError::type_error(e.0)
    }
}

// ── Shared stub message ───────────────────────────────────────────────────────

const STUB_MSG: &str = "CPython bridge requires the 'cpython' feature and Python 3.x installed. \
     Rebuild with: cargo build --features unilang-cpython/cpython";

// ─────────────────────────────────────────────────────────────────────────────
// CPython-enabled implementation (PyO3)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "cpython")]
mod cpython_impl {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList, PyTuple};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use super::BridgeError;
    use unilang_runtime::value::RuntimeValue;

    /// Shared state holding imported Python modules.
    #[derive(Default, Clone)]
    pub struct BridgeState {
        /// Maps module name → imported PyObject (held across calls).
        modules: HashMap<String, PyObject>,
    }

    impl BridgeState {
        pub fn new() -> Self {
            Self::default()
        }
    }

    /// Shared, mutex-protected bridge state.
    pub type SharedState = Arc<Mutex<BridgeState>>;

    // ── RuntimeValue ↔ Python object conversions ──────────────────────────────

    /// Convert a `RuntimeValue` to a Python object.
    pub fn rv_to_py(py: Python<'_>, rv: &RuntimeValue) -> PyResult<PyObject> {
        match rv {
            RuntimeValue::Int(n) => Ok(n.to_object(py)),
            RuntimeValue::Float(f) => Ok(f.to_object(py)),
            RuntimeValue::Bool(b) => Ok(b.to_object(py)),
            RuntimeValue::String(s) => Ok(s.to_object(py)),
            RuntimeValue::Null => Ok(py.None()),
            RuntimeValue::List(items) => {
                let list = PyList::empty(py);
                for item in items {
                    list.append(rv_to_py(py, item)?)?;
                }
                Ok(list.to_object(py))
            }
            RuntimeValue::Dict(pairs) => {
                let dict = PyDict::new(py);
                for (k, v) in pairs {
                    dict.set_item(rv_to_py(py, k)?, rv_to_py(py, v)?)?;
                }
                Ok(dict.to_object(py))
            }
            // Function / Instance / NativeFunction / Class cannot be passed to Python.
            other => Ok(other.to_string().to_object(py)),
        }
    }

    /// Convert a Python object to a `RuntimeValue`.
    pub fn py_to_rv(py: Python<'_>, obj: &PyObject) -> RuntimeValue {
        let bound = obj.bind(py);

        // Try bool first (Python bool is a subclass of int).
        if let Ok(b) = bound.extract::<bool>() {
            return RuntimeValue::Bool(b);
        }
        if let Ok(n) = bound.extract::<i64>() {
            return RuntimeValue::Int(n);
        }
        if let Ok(f) = bound.extract::<f64>() {
            return RuntimeValue::Float(f);
        }
        if let Ok(s) = bound.extract::<String>() {
            return RuntimeValue::String(s);
        }
        if bound.is_none() {
            return RuntimeValue::Null;
        }
        // List / tuple.
        if let Ok(list) = bound.downcast::<PyList>() {
            let items: Vec<RuntimeValue> = list
                .iter()
                .map(|item| py_to_rv(py, &item.to_object(py)))
                .collect();
            return RuntimeValue::List(items);
        }
        if let Ok(tup) = bound.downcast::<PyTuple>() {
            let items: Vec<RuntimeValue> = tup
                .iter()
                .map(|item| py_to_rv(py, &item.to_object(py)))
                .collect();
            return RuntimeValue::List(items);
        }
        // Dict.
        if let Ok(dict) = bound.downcast::<PyDict>() {
            let pairs: Vec<(RuntimeValue, RuntimeValue)> = dict
                .items()
                .iter()
                .map(|pair| {
                    let kv = pair
                        .downcast::<PyTuple>()
                        .expect("dict item should be tuple");
                    let k = kv.get_item(0).unwrap().to_object(py);
                    let v = kv.get_item(1).unwrap().to_object(py);
                    (py_to_rv(py, &k), py_to_rv(py, &v))
                })
                .collect();
            return RuntimeValue::Dict(pairs);
        }
        // Fallback: string representation.
        RuntimeValue::String(obj.to_string())
    }

    // ── Public bridge functions ───────────────────────────────────────────────

    /// Import a Python module and store it in the bridge state.
    pub fn py_import(state: &SharedState, module_name: &str) -> Result<RuntimeValue, BridgeError> {
        Python::with_gil(|py| {
            let module = py
                .import(module_name)
                .map_err(|e| BridgeError(format!("py_import('{}') failed: {}", module_name, e)))?;
            let mut guard = state
                .lock()
                .map_err(|e| BridgeError(format!("bridge state lock poisoned: {}", e)))?;
            guard
                .modules
                .insert(module_name.to_string(), module.to_object(py));
            Ok(RuntimeValue::Bool(true))
        })
    }

    /// Call a function from a previously imported module.
    pub fn py_call(
        state: &SharedState,
        module_name: &str,
        func_name: &str,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue, BridgeError> {
        Python::with_gil(|py| {
            let module_obj = {
                let guard = state
                    .lock()
                    .map_err(|e| BridgeError(format!("lock poisoned: {}", e)))?;
                guard.modules.get(module_name).cloned().ok_or_else(|| {
                    BridgeError(format!(
                        "module '{}' not imported; call py_import('{}') first",
                        module_name, module_name
                    ))
                })?
            };

            let py_args: Vec<PyObject> = args
                .iter()
                .map(|a| rv_to_py(py, a))
                .collect::<PyResult<Vec<_>>>()
                .map_err(|e| BridgeError(format!("argument conversion failed: {}", e)))?;

            let func = module_obj.bind(py).getattr(func_name).map_err(|e| {
                BridgeError(format!(
                    "module '{}' has no attribute '{}': {}",
                    module_name, func_name, e
                ))
            })?;

            let tuple_args = PyTuple::new(py, py_args.iter());
            let result = func
                .call1(tuple_args)
                .map_err(|e| {
                    BridgeError(format!(
                        "py_call({}.{}) failed: {}",
                        module_name, func_name, e
                    ))
                })?
                .to_object(py);

            Ok(py_to_rv(py, &result))
        })
    }

    /// Evaluate a Python expression.
    pub fn py_eval(expr: &str) -> Result<RuntimeValue, BridgeError> {
        Python::with_gil(|py| {
            let result = py
                .eval(expr, None, None)
                .map_err(|e| BridgeError(format!("py_eval('{}') failed: {}", expr, e)))?
                .to_object(py);
            Ok(py_to_rv(py, &result))
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stub implementation (no cpython feature)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(not(feature = "cpython"))]
mod stub_impl {
    use super::{BridgeError, STUB_MSG};
    use unilang_runtime::value::RuntimeValue;

    pub fn py_import(_module_name: &str) -> Result<RuntimeValue, BridgeError> {
        Err(BridgeError(STUB_MSG.to_string()))
    }

    pub fn py_call(
        _module: &str,
        _func: &str,
        _args: &[RuntimeValue],
    ) -> Result<RuntimeValue, BridgeError> {
        Err(BridgeError(STUB_MSG.to_string()))
    }

    pub fn py_eval(_expr: &str) -> Result<RuntimeValue, BridgeError> {
        Err(BridgeError(STUB_MSG.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API (delegates to the appropriate implementation)
// ─────────────────────────────────────────────────────────────────────────────

// Single shared thread-local bridge state (cpython feature only).
// All three public functions share this state so that modules imported via
// `py_import` are visible to subsequent `py_call` invocations.
#[cfg(feature = "cpython")]
thread_local! {
    static BRIDGE_STATE: std::sync::Arc<std::sync::Mutex<cpython_impl::BridgeState>> =
        std::sync::Arc::new(std::sync::Mutex::new(cpython_impl::BridgeState::new()));
}

/// Import a Python module by name.
///
/// On success returns `RuntimeValue::Bool(true)`.
/// Requires the `cpython` feature.
pub fn py_import(module_name: &str) -> Result<RuntimeValue, BridgeError> {
    #[cfg(feature = "cpython")]
    {
        BRIDGE_STATE.with(|s| cpython_impl::py_import(s, module_name))
    }
    #[cfg(not(feature = "cpython"))]
    stub_impl::py_import(module_name)
}

/// Call a function from a previously imported Python module.
///
/// Requires the `cpython` feature and a prior `py_import` call for the module.
pub fn py_call(
    module_name: &str,
    func_name: &str,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, BridgeError> {
    #[cfg(feature = "cpython")]
    {
        BRIDGE_STATE.with(|s| cpython_impl::py_call(s, module_name, func_name, args))
    }
    #[cfg(not(feature = "cpython"))]
    stub_impl::py_call(module_name, func_name, args)
}

/// Evaluate a Python expression and return the result.
///
/// Requires the `cpython` feature.
pub fn py_eval(expr: &str) -> Result<RuntimeValue, BridgeError> {
    #[cfg(feature = "cpython")]
    {
        cpython_impl::py_eval(expr)
    }
    #[cfg(not(feature = "cpython"))]
    stub_impl::py_eval(expr)
}

// ─────────────────────────────────────────────────────────────────────────────
// VM builtin registration
// ─────────────────────────────────────────────────────────────────────────────

/// Register `py_import`, `py_call`, and `py_eval` as UniLang VM builtins.
///
/// Call this after creating your `VM`:
/// ```rust,ignore
/// let mut vm = unilang_runtime::vm::VM::new();
/// unilang_stdlib::register_builtins(&mut vm);
/// unilang_cpython::register_builtins(&mut vm);
/// ```
pub fn register_builtins(vm: &mut VM) {
    // ── py_import(module_name) ────────────────────────────────────────────────
    vm.register_builtin("py_import", |args| {
        let module_name = require_string(args, 0, "py_import(module_name)")?;
        py_import(&module_name).map_err(|e| RuntimeError::type_error(e.0))
    });

    // ── py_call(module, func, arg0, arg1, ...) ────────────────────────────────
    vm.register_builtin("py_call", |args| {
        if args.len() < 2 {
            return Err(RuntimeError::type_error(
                "py_call(module, func, ...args) requires at least 2 arguments",
            ));
        }
        let module_name = require_string(args, 0, "py_call")?;
        let func_name = require_string(args, 1, "py_call")?;
        let call_args = &args[2..];
        py_call(&module_name, &func_name, call_args).map_err(|e| RuntimeError::type_error(e.0))
    });

    // ── py_eval(expr) ─────────────────────────────────────────────────────────
    vm.register_builtin("py_eval", |args| {
        let expr = require_string(args, 0, "py_eval(expr)")?;
        py_eval(&expr).map_err(|e| RuntimeError::type_error(e.0))
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn require_string(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        Some(other) => Ok(format!("{}", other)),
        None => Err(RuntimeError::type_error(format!(
            "{}: missing argument at position {}",
            sig, idx
        ))),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_py_import_returns_error() {
        // In default (no cpython feature) mode this should return a stub error.
        let result = py_import("numpy");
        #[cfg(not(feature = "cpython"))]
        assert!(result.is_err());
        #[cfg(feature = "cpython")]
        let _ = result; // may succeed or fail depending on environment
    }

    #[test]
    fn stub_py_eval_returns_error() {
        let result = py_eval("1 + 1");
        #[cfg(not(feature = "cpython"))]
        assert!(result.is_err());
        #[cfg(feature = "cpython")]
        {
            // If cpython feature is on AND Python is available this should return Int(2).
            let _ = result;
        }
    }

    #[test]
    fn stub_py_call_returns_error() {
        let result = py_call("math", "sqrt", &[RuntimeValue::Float(4.0)]);
        #[cfg(not(feature = "cpython"))]
        assert!(result.is_err());
        #[cfg(feature = "cpython")]
        let _ = result;
    }

    #[test]
    fn register_builtins_smoke() {
        let mut vm = unilang_runtime::vm::VM::new();
        // Should not panic.
        register_builtins(&mut vm);
    }
}
