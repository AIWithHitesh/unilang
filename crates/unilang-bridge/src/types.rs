// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Type marshaling between UniLang RuntimeValue and cross-VM bridge values.
//!
//! This module defines [`BridgeValue`], which mirrors [`unilang_runtime::value::RuntimeValue`]
//! but includes variants for opaque JVM and CPython object handles. The conversion functions
//! [`runtime_to_bridge`] and [`bridge_to_runtime`] perform real bidirectional conversions.

use unilang_runtime::value::RuntimeValue;

#[cfg(any(feature = "jvm", feature = "cpython"))]
use crate::error::BridgeError;

/// A value that can be passed across VM boundaries (JVM or CPython).
///
/// This mirrors `RuntimeValue` but adds opaque handle variants for foreign objects
/// that live in a JVM or CPython heap and cannot be represented as native Rust values.
#[derive(Debug, Clone)]
pub enum BridgeValue {
    /// The null/None value.
    Null,
    /// A boolean value.
    Bool(bool),
    /// A 64-bit signed integer.
    Int(i64),
    /// A 64-bit floating-point number.
    Float(f64),
    /// A UTF-8 string.
    String(String),
    /// An ordered list of bridge values.
    List(Vec<BridgeValue>),
    /// An ordered map of string keys to bridge values.
    Dict(Vec<(String, BridgeValue)>),
    /// An opaque reference to a live JVM object.
    JavaObject {
        /// Fully-qualified class name (e.g. `"java.util.ArrayList"`).
        class: String,
        /// Opaque handle into the JVM object table managed by the JNI layer.
        handle: u64,
    },
    /// An opaque reference to a live CPython object.
    PythonObject {
        /// The module the object originates from (e.g. `"collections"`).
        module: String,
        /// The type/class name of the object (e.g. `"OrderedDict"`).
        name: String,
        /// Opaque handle into the CPython object table managed by the bridge layer.
        handle: u64,
    },
}

/// Convert a [`RuntimeValue`] into a [`BridgeValue`].
///
/// Primitive types (Int, Float, String, Bool, Null, List, Dict) are converted
/// directly. Instance, Function, NativeFunction and Class values are represented
/// as opaque strings.
pub fn runtime_to_bridge(v: &RuntimeValue) -> BridgeValue {
    match v {
        RuntimeValue::Null => BridgeValue::Null,
        RuntimeValue::Bool(b) => BridgeValue::Bool(*b),
        RuntimeValue::Int(n) => BridgeValue::Int(*n),
        RuntimeValue::Float(f) => BridgeValue::Float(*f),
        RuntimeValue::String(s) => BridgeValue::String(s.clone()),
        RuntimeValue::List(items) => {
            BridgeValue::List(items.iter().map(runtime_to_bridge).collect())
        }
        RuntimeValue::Dict(pairs) => {
            let converted = pairs
                .iter()
                .map(|(k, v)| {
                    let key = match k {
                        RuntimeValue::String(s) => s.clone(),
                        other => format!("{}", other),
                    };
                    (key, runtime_to_bridge(v))
                })
                .collect();
            BridgeValue::Dict(converted)
        }
        RuntimeValue::Function(idx) => BridgeValue::String(format!("<function {}>", idx)),
        RuntimeValue::Instance(data) => {
            BridgeValue::String(format!("<{} instance>", data.class_name))
        }
        RuntimeValue::NativeFunction(name) => BridgeValue::String(format!("<builtin {}>", name)),
        RuntimeValue::Class(name) => BridgeValue::String(format!("<class {}>", name)),
    }
}

/// Convert a [`BridgeValue`] back into a [`RuntimeValue`].
///
/// `JavaObject` and `PythonObject` handles are rendered as string descriptors
/// since they reference heap objects in a foreign VM that cannot be moved into
/// the UniLang runtime.
pub fn bridge_to_runtime(v: BridgeValue) -> RuntimeValue {
    match v {
        BridgeValue::Null => RuntimeValue::Null,
        BridgeValue::Bool(b) => RuntimeValue::Bool(b),
        BridgeValue::Int(n) => RuntimeValue::Int(n),
        BridgeValue::Float(f) => RuntimeValue::Float(f),
        BridgeValue::String(s) => RuntimeValue::String(s),
        BridgeValue::List(items) => {
            RuntimeValue::List(items.into_iter().map(bridge_to_runtime).collect())
        }
        BridgeValue::Dict(pairs) => {
            let converted = pairs
                .into_iter()
                .map(|(k, v)| (RuntimeValue::String(k), bridge_to_runtime(v)))
                .collect();
            RuntimeValue::Dict(converted)
        }
        BridgeValue::JavaObject { class, handle } => {
            RuntimeValue::String(format!("<JavaObject:{}:{}>", class, handle))
        }
        BridgeValue::PythonObject {
            module,
            name,
            handle,
        } => RuntimeValue::String(format!("<PythonObject:{}.{}:{}>", module, name, handle)),
    }
}

// ── JVM-specific conversions ──────────────────────────────────────────────────

#[cfg(feature = "jvm")]
pub use jvm_marshal::{bridge_to_jvalue, jvalue_to_bridge};

#[cfg(feature = "jvm")]
mod jvm_marshal {
    use jni::objects::{JObject, JString, JValueOwned};
    use jni::JNIEnv;

    use super::BridgeValue;
    use crate::error::BridgeError;

    /// Convert a JVM [`JValueOwned`] into a [`BridgeValue`].
    pub fn jvalue_to_bridge(env: &JNIEnv, val: JValueOwned) -> Result<BridgeValue, BridgeError> {
        match val {
            JValueOwned::Bool(b) => Ok(BridgeValue::Bool(b != 0)),
            JValueOwned::Byte(b) => Ok(BridgeValue::Int(b as i64)),
            JValueOwned::Char(c) => Ok(BridgeValue::String(
                char::from_u32(c as u32)
                    .map(|ch| ch.to_string())
                    .unwrap_or_else(|| format!("\\u{:04x}", c)),
            )),
            JValueOwned::Short(s) => Ok(BridgeValue::Int(s as i64)),
            JValueOwned::Int(i) => Ok(BridgeValue::Int(i as i64)),
            JValueOwned::Long(l) => Ok(BridgeValue::Int(l)),
            JValueOwned::Float(f) => Ok(BridgeValue::Float(f as f64)),
            JValueOwned::Double(d) => Ok(BridgeValue::Float(d)),
            JValueOwned::Void => Ok(BridgeValue::Null),
            JValueOwned::Object(obj) => {
                if obj.is_null() {
                    return Ok(BridgeValue::Null);
                }
                // Try to treat it as a String first.
                let jstring: JString = JString::from(obj);
                match env.get_string(&jstring) {
                    Ok(java_str) => {
                        let s: String = java_str.into();
                        Ok(BridgeValue::String(s))
                    }
                    Err(_) => {
                        // Return as a JavaObject with a placeholder handle.
                        Ok(BridgeValue::JavaObject {
                            class: "<unknown>".to_string(),
                            handle: 0,
                        })
                    }
                }
            }
        }
    }

    /// Convert a [`BridgeValue`] into a JVM [`JValueOwned`] suitable for passing
    /// to JNI method-call functions.
    pub fn bridge_to_jvalue<'a>(
        env: &mut JNIEnv<'a>,
        val: &BridgeValue,
    ) -> Result<JValueOwned<'a>, BridgeError> {
        match val {
            BridgeValue::Null => Ok(JValueOwned::Object(JObject::null())),
            BridgeValue::Bool(b) => Ok(JValueOwned::Bool(if *b { 1 } else { 0 })),
            BridgeValue::Int(n) => Ok(JValueOwned::Long(*n)),
            BridgeValue::Float(f) => Ok(JValueOwned::Double(*f)),
            BridgeValue::String(s) => {
                let jstr = env
                    .new_string(s)
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                Ok(JValueOwned::Object(jstr.into()))
            }
            BridgeValue::List(items) => {
                // Convert list to Object[] array.
                let arr = env
                    .new_object_array(items.len() as i32, "java/lang/Object", JObject::null())
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                for (i, item) in items.iter().enumerate() {
                    let jval = bridge_to_jvalue(env, item)?;
                    let obj = jval_to_jobject(env, jval)?;
                    env.set_object_array_element(&arr, i as i32, obj)
                        .map_err(|e| BridgeError::from_jni(e.into()))?;
                }
                Ok(JValueOwned::Object(arr.into()))
            }
            BridgeValue::Dict(_) => {
                // Represent dict as a serialized string for simplicity.
                let repr = format!("{:?}", val);
                let jstr = env
                    .new_string(&repr)
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                Ok(JValueOwned::Object(jstr.into()))
            }
            BridgeValue::JavaObject { handle, .. } => {
                // The caller is responsible for passing the raw handle back into
                // the bridge for lookups; here we return null as a safe fallback.
                let _ = handle;
                Ok(JValueOwned::Object(JObject::null()))
            }
            BridgeValue::PythonObject { .. } => Err(BridgeError::MarshalingError(
                "cannot convert PythonObject to JValue".to_string(),
            )),
        }
    }

    /// Helper: box a primitive JValueOwned into a java.lang.* wrapper object.
    fn jval_to_jobject<'a>(
        env: &mut JNIEnv<'a>,
        val: JValueOwned<'a>,
    ) -> Result<JObject<'a>, BridgeError> {
        match val {
            JValueOwned::Object(obj) => Ok(obj),
            JValueOwned::Bool(b) => {
                let cls = env
                    .find_class("java/lang/Boolean")
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                let obj = env
                    .call_static_method(
                        cls,
                        "valueOf",
                        "(Z)Ljava/lang/Boolean;",
                        &[JValueOwned::Bool(b).borrow()],
                    )
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                Ok(obj.l().map_err(|e| BridgeError::from_jni(e.into()))?)
            }
            JValueOwned::Long(l) => {
                let cls = env
                    .find_class("java/lang/Long")
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                let obj = env
                    .call_static_method(
                        cls,
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[JValueOwned::Long(l).borrow()],
                    )
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                Ok(obj.l().map_err(|e| BridgeError::from_jni(e.into()))?)
            }
            JValueOwned::Double(d) => {
                let cls = env
                    .find_class("java/lang/Double")
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                let obj = env
                    .call_static_method(
                        cls,
                        "valueOf",
                        "(D)Ljava/lang/Double;",
                        &[JValueOwned::Double(d).borrow()],
                    )
                    .map_err(|e| BridgeError::from_jni(e.into()))?;
                Ok(obj.l().map_err(|e| BridgeError::from_jni(e.into()))?)
            }
            other => Err(BridgeError::MarshalingError(format!(
                "cannot box JValue variant {:?} to Object",
                other
            ))),
        }
    }
}

// ── CPython-specific conversions ──────────────────────────────────────────────

#[cfg(feature = "cpython")]
pub use cpython_marshal::{bridge_to_pyobject, pyobject_to_bridge};

#[cfg(feature = "cpython")]
mod cpython_marshal {
    use pyo3::prelude::*;
    use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};

    use super::BridgeValue;
    use crate::error::BridgeError;

    /// Convert a Python [`PyAny`] object into a [`BridgeValue`].
    pub fn pyobject_to_bridge(
        py: Python<'_>,
        obj: &Bound<'_, PyAny>,
    ) -> Result<BridgeValue, BridgeError> {
        if obj.is_none() {
            return Ok(BridgeValue::Null);
        }
        if let Ok(b) = obj.downcast::<PyBool>() {
            return Ok(BridgeValue::Bool(b.is_true()));
        }
        if let Ok(i) = obj.downcast::<PyInt>() {
            let val: i64 = i.extract().map_err(BridgeError::from_pyo3)?;
            return Ok(BridgeValue::Int(val));
        }
        if let Ok(f) = obj.downcast::<PyFloat>() {
            let val: f64 = f.extract().map_err(BridgeError::from_pyo3)?;
            return Ok(BridgeValue::Float(val));
        }
        if let Ok(s) = obj.downcast::<PyString>() {
            let val: String = s.extract().map_err(BridgeError::from_pyo3)?;
            return Ok(BridgeValue::String(val));
        }
        if let Ok(lst) = obj.downcast::<PyList>() {
            let items: Result<Vec<BridgeValue>, BridgeError> = lst
                .iter()
                .map(|item| pyobject_to_bridge(py, &item))
                .collect();
            return Ok(BridgeValue::List(items?));
        }
        if let Ok(dct) = obj.downcast::<PyDict>() {
            let mut pairs = Vec::new();
            for (k, v) in dct.iter() {
                let key: String = k.extract().map_err(BridgeError::from_pyo3)?;
                let val = pyobject_to_bridge(py, &v)?;
                pairs.push((key, val));
            }
            return Ok(BridgeValue::Dict(pairs));
        }
        // Fallback: represent as string via __repr__.
        let repr: String = obj
            .repr()
            .and_then(|r| r.extract())
            .map_err(BridgeError::from_pyo3)?;
        Ok(BridgeValue::String(repr))
    }

    /// Convert a [`BridgeValue`] into a Python [`PyObject`].
    pub fn bridge_to_pyobject(py: Python<'_>, val: &BridgeValue) -> Result<PyObject, BridgeError> {
        match val {
            BridgeValue::Null => Ok(PyNone::get(py).into()),
            BridgeValue::Bool(b) => Ok(b.into_pyobject(py).map_err(BridgeError::from_pyo3)?.into()),
            BridgeValue::Int(n) => Ok(n.into_pyobject(py).map_err(BridgeError::from_pyo3)?.into()),
            BridgeValue::Float(f) => {
                Ok(f.into_pyobject(py).map_err(BridgeError::from_pyo3)?.into())
            }
            BridgeValue::String(s) => Ok(s
                .clone()
                .into_pyobject(py)
                .map_err(BridgeError::from_pyo3)?
                .into()),
            BridgeValue::List(items) => {
                let py_items: Result<Vec<PyObject>, BridgeError> = items
                    .iter()
                    .map(|item| bridge_to_pyobject(py, item))
                    .collect();
                let list = PyList::new(py, py_items?).map_err(BridgeError::from_pyo3)?;
                Ok(list.into())
            }
            BridgeValue::Dict(pairs) => {
                let dict = PyDict::new(py);
                for (k, v) in pairs {
                    let py_val = bridge_to_pyobject(py, v)?;
                    dict.set_item(k, py_val).map_err(BridgeError::from_pyo3)?;
                }
                Ok(dict.into())
            }
            BridgeValue::JavaObject { class, handle } => {
                let s = format!("<JavaObject:{}:{}>", class, handle);
                Ok(s.into_pyobject(py).map_err(BridgeError::from_pyo3)?.into())
            }
            BridgeValue::PythonObject {
                module,
                name,
                handle,
            } => {
                let s = format!("<PythonObject:{}.{}:{}>", module, name, handle);
                Ok(s.into_pyobject(py).map_err(BridgeError::from_pyo3)?.into())
            }
        }
    }
}
