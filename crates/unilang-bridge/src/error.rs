// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Bridge error types for JVM and CPython interop.

use std::fmt;

/// Errors that can occur during JVM or CPython bridge operations.
#[derive(Debug, Clone)]
pub enum BridgeError {
    /// The JVM bridge is not available (feature not enabled or JVM not installed).
    JvmNotAvailable(String),
    /// The CPython bridge is not available (feature not enabled or Python headers missing).
    CpythonNotAvailable(String),
    /// A type marshaling error occurred when converting values across VM boundaries.
    MarshalingError(String),
    /// An exception was raised by the remote VM (JVM or CPython).
    CrossVmException {
        /// The source VM or location where the exception originated.
        source: String,
        /// Human-readable description of the exception.
        message: String,
    },
}

impl BridgeError {
    /// Construct a `CrossVmException` from a JNI error (only compiled with `jvm` feature).
    #[cfg(feature = "jvm")]
    pub fn from_jni(err: jni::errors::Error) -> Self {
        BridgeError::CrossVmException {
            source: "JVM".to_string(),
            message: err.to_string(),
        }
    }

    /// Construct a `CrossVmException` from a pyo3 `PyErr` (only compiled with `cpython` feature).
    #[cfg(feature = "cpython")]
    pub fn from_pyo3(err: pyo3::PyErr) -> Self {
        BridgeError::CrossVmException {
            source: "CPython".to_string(),
            message: err.to_string(),
        }
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeError::JvmNotAvailable(msg) => {
                write!(f, "JVM bridge not available: {}", msg)
            }
            BridgeError::CpythonNotAvailable(msg) => {
                write!(f, "CPython bridge not available: {}", msg)
            }
            BridgeError::MarshalingError(msg) => {
                write!(f, "marshaling error: {}", msg)
            }
            BridgeError::CrossVmException { source, message } => {
                write!(f, "cross-VM exception from {}: {}", source, message)
            }
        }
    }
}

impl std::error::Error for BridgeError {}
