// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! CPython bridge implementation for UniLang v2.0 interop.
//!
//! With the `cpython` feature enabled this module manages a pyo3 interpreter,
//! allowing UniLang programs to import modules, call functions, evaluate
//! expressions, and execute statements.
//!
//! Without the `cpython` feature every method returns
//! [`BridgeError::CpythonNotAvailable`].

use crate::error::BridgeError;
use crate::types::BridgeValue;

/// A handle to an active CPython interpreter session.
pub struct CpythonBridge {
    #[cfg(feature = "cpython")]
    modules: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, pyo3::PyObject>>>,
    #[cfg(feature = "cpython")]
    objects: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, pyo3::PyObject>>>,
    #[cfg(feature = "cpython")]
    next_handle: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

unsafe impl Send for CpythonBridge {}
unsafe impl Sync for CpythonBridge {}

impl CpythonBridge {
    /// Initialise the CPython interpreter tables.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError::CpythonNotAvailable`] when the `cpython` feature is disabled.
    pub fn new() -> Result<Self, BridgeError> {
        #[cfg(feature = "cpython")]
        {
            Ok(Self {
                modules: std::sync::Arc::new(std::sync::Mutex::new(
                    std::collections::HashMap::new(),
                )),
                objects: std::sync::Arc::new(std::sync::Mutex::new(
                    std::collections::HashMap::new(),
                )),
                next_handle: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Import a Python module and return an opaque module handle.
    pub fn import_module(&self, module: &str) -> Result<u64, BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use pyo3::prelude::*;
            use pyo3::types::PyModule;

            let py_obj = Python::with_gil(|py| -> Result<PyObject, BridgeError> {
                let m = PyModule::import(py, module).map_err(BridgeError::from_pyo3)?;
                Ok(m.into())
            })?;

            let handle = self
                .next_handle
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.modules
                .lock()
                .map_err(|_| BridgeError::MarshalingError("module table mutex poisoned".into()))?
                .insert(handle, py_obj);
            Ok(handle)
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = module;
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Call a top-level function in an already-imported module.
    pub fn call_function(
        &self,
        module_handle: u64,
        func: &str,
        args: &[BridgeValue],
    ) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use crate::types::{bridge_to_pyobject, pyobject_to_bridge};
            use pyo3::prelude::*;
            use pyo3::types::PyTuple;

            let module_obj = {
                let table = self.modules.lock().map_err(|_| {
                    BridgeError::MarshalingError("module table mutex poisoned".into())
                })?;
                table.get(&module_handle).cloned().ok_or_else(|| {
                    BridgeError::MarshalingError(format!("no module with handle {}", module_handle))
                })?
            };

            Python::with_gil(|py| -> Result<BridgeValue, BridgeError> {
                let m = module_obj.bind(py);
                let fn_obj = m.getattr(func).map_err(BridgeError::from_pyo3)?;

                let py_args: Result<Vec<PyObject>, BridgeError> =
                    args.iter().map(|a| bridge_to_pyobject(py, a)).collect();
                let tuple = PyTuple::new(py, py_args?).map_err(BridgeError::from_pyo3)?;

                let result = fn_obj.call1(tuple).map_err(BridgeError::from_pyo3)?;

                pyobject_to_bridge(py, &result)
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = (module_handle, func, args);
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Call a method on a live Python object identified by its handle.
    pub fn call_method(
        &self,
        obj_handle: u64,
        method: &str,
        args: &[BridgeValue],
    ) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use crate::types::{bridge_to_pyobject, pyobject_to_bridge};
            use pyo3::prelude::*;
            use pyo3::types::PyTuple;

            let obj = {
                let table = self.objects.lock().map_err(|_| {
                    BridgeError::MarshalingError("object table mutex poisoned".into())
                })?;
                table.get(&obj_handle).cloned().ok_or_else(|| {
                    BridgeError::MarshalingError(format!("no object with handle {}", obj_handle))
                })?
            };

            Python::with_gil(|py| -> Result<BridgeValue, BridgeError> {
                let bound = obj.bind(py);
                let py_args: Result<Vec<PyObject>, BridgeError> =
                    args.iter().map(|a| bridge_to_pyobject(py, a)).collect();
                let tuple = PyTuple::new(py, py_args?).map_err(BridgeError::from_pyo3)?;

                let result = bound
                    .call_method1(method, tuple)
                    .map_err(BridgeError::from_pyo3)?;

                pyobject_to_bridge(py, &result)
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = (obj_handle, method, args);
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Read an attribute from a live Python object.
    pub fn get_attribute(&self, obj_handle: u64, attr: &str) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use crate::types::pyobject_to_bridge;
            use pyo3::prelude::*;

            let obj = {
                let table = self.objects.lock().map_err(|_| {
                    BridgeError::MarshalingError("object table mutex poisoned".into())
                })?;
                table.get(&obj_handle).cloned().ok_or_else(|| {
                    BridgeError::MarshalingError(format!("no object with handle {}", obj_handle))
                })?
            };

            Python::with_gil(|py| -> Result<BridgeValue, BridgeError> {
                let bound = obj.bind(py);
                let attr_val = bound.getattr(attr).map_err(BridgeError::from_pyo3)?;
                pyobject_to_bridge(py, &attr_val)
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = (obj_handle, attr);
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Evaluate a Python expression and return its value.
    pub fn eval(&self, code: &str) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use crate::types::pyobject_to_bridge;
            use pyo3::prelude::*;

            Python::with_gil(|py| -> Result<BridgeValue, BridgeError> {
                let result = py
                    .eval(pyo3::ffi::c_str!(code), None, None)
                    .map_err(BridgeError::from_pyo3)?;
                pyobject_to_bridge(py, &result)
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = code;
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Execute a block of Python statements.
    pub fn exec(&self, code: &str) -> Result<(), BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use pyo3::prelude::*;

            Python::with_gil(|py| -> Result<(), BridgeError> {
                py.run(pyo3::ffi::c_str!(code), None, None)
                    .map_err(BridgeError::from_pyo3)
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = code;
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Append a directory to `sys.path`.
    pub fn add_to_sys_path(&self, path: &str) -> Result<(), BridgeError> {
        #[cfg(feature = "cpython")]
        {
            use pyo3::prelude::*;
            use pyo3::types::PyModule;

            Python::with_gil(|py| -> Result<(), BridgeError> {
                let sys = PyModule::import(py, "sys").map_err(BridgeError::from_pyo3)?;
                let sys_path = sys.getattr("path").map_err(BridgeError::from_pyo3)?;
                sys_path
                    .call_method1("append", (path,))
                    .map_err(BridgeError::from_pyo3)?;
                Ok(())
            })
        }
        #[cfg(not(feature = "cpython"))]
        {
            let _ = path;
            Err(BridgeError::CpythonNotAvailable(
                "compile with '--features cpython' to enable the CPython bridge".to_string(),
            ))
        }
    }

    /// Store an arbitrary Python object in the object table and return its handle.
    #[cfg(feature = "cpython")]
    pub fn store_object(&self, obj: pyo3::PyObject) -> u64 {
        let handle = self
            .next_handle
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Ok(mut table) = self.objects.lock() {
            table.insert(handle, obj);
        }
        handle
    }
}
