// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Zero-copy array bridge: JVM `DirectByteBuffer` ↔ NumPy buffer protocol.
//!
//! [`SharedArrayBuffer`] holds a typed array in Rust-owned memory and can expose
//! it to a JVM byte array or a NumPy `ndarray` without an extra copy.

#[cfg(any(feature = "jvm", feature = "cpython"))]
use crate::error::BridgeError;

/// Numeric element type of a [`SharedArrayBuffer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayDtype {
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
}

impl ArrayDtype {
    /// Size in bytes of a single element.
    pub fn element_size(&self) -> usize {
        match self {
            ArrayDtype::Int8 => 1,
            ArrayDtype::Int16 => 2,
            ArrayDtype::Int32 => 4,
            ArrayDtype::Int64 => 8,
            ArrayDtype::Float32 => 4,
            ArrayDtype::Float64 => 8,
        }
    }

    /// JVM array type descriptor, e.g. `"[B"` for `byte[]`.
    pub fn java_type_sig(&self) -> &'static str {
        match self {
            ArrayDtype::Int8 => "[B",
            ArrayDtype::Int16 => "[S",
            ArrayDtype::Int32 => "[I",
            ArrayDtype::Int64 => "[J",
            ArrayDtype::Float32 => "[F",
            ArrayDtype::Float64 => "[D",
        }
    }

    /// NumPy dtype string, e.g. `"float64"`.
    pub fn numpy_dtype(&self) -> &'static str {
        match self {
            ArrayDtype::Int8 => "int8",
            ArrayDtype::Int16 => "int16",
            ArrayDtype::Int32 => "int32",
            ArrayDtype::Int64 => "int64",
            ArrayDtype::Float32 => "float32",
            ArrayDtype::Float64 => "float64",
        }
    }
}

/// A typed, heap-allocated array that can be shared across VM boundaries.
///
/// In a production zero-copy implementation `data` would be pinned memory
/// accessible through both a JVM `DirectByteBuffer` and a NumPy array via the
/// buffer protocol. The current implementation uses an owned `Vec<u8>` for
/// portability and safety.
pub struct SharedArrayBuffer {
    /// Raw byte storage (little-endian).
    pub data: Vec<u8>,
    /// Element type.
    pub dtype: ArrayDtype,
    /// Shape (number of elements per dimension).
    pub shape: Vec<usize>,
}

impl SharedArrayBuffer {
    /// Allocate a zero-filled buffer with the given dtype and shape.
    pub fn new(dtype: ArrayDtype, shape: Vec<usize>) -> Self {
        let elem_count: usize = shape.iter().product();
        let byte_count = elem_count * dtype.element_size();
        Self {
            data: vec![0u8; byte_count],
            dtype,
            shape,
        }
    }

    /// Total number of elements (product of shape dimensions).
    pub fn len(&self) -> usize {
        self.shape.iter().product()
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total size in bytes.
    pub fn byte_len(&self) -> usize {
        self.len() * self.dtype.element_size()
    }

    /// Copy the buffer contents into a JVM byte array.
    ///
    /// The raw bytes are treated as a flat `byte[]` regardless of dtype; the
    /// caller is responsible for interpreting the byte layout on the Java side
    /// (e.g. via `ByteBuffer.wrap(...).order(ByteOrder.LITTLE_ENDIAN)`).
    #[cfg(feature = "jvm")]
    pub fn to_java_array(
        &self,
        env: &mut jni::JNIEnv,
    ) -> Result<jni::objects::JByteArray, BridgeError> {
        use jni::objects::JByteArray;

        let arr: JByteArray = env
            .new_byte_array(self.data.len() as i32)
            .map_err(|e| BridgeError::from_jni(e.into()))?;

        // SAFETY: i8 and u8 have the same layout; JNI uses signed bytes.
        let signed: &[i8] =
            unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const i8, self.data.len()) };

        env.set_byte_array_region(&arr, 0, signed)
            .map_err(|e| BridgeError::from_jni(e.into()))?;

        Ok(arr)
    }

    /// Wrap the buffer as a NumPy `ndarray` using the `numpy` Python module.
    ///
    /// The array is created from a copy of the byte data; modifying the NumPy
    /// array does not affect the Rust buffer.
    #[cfg(feature = "cpython")]
    pub fn to_numpy_array(&self, py: pyo3::Python<'_>) -> Result<pyo3::PyObject, BridgeError> {
        use pyo3::prelude::*;
        use pyo3::types::{PyBytes, PyModule};

        let np = PyModule::import(py, "numpy").map_err(BridgeError::from_pyo3)?;
        let py_bytes = PyBytes::new(py, &self.data);

        // numpy.frombuffer(bytes, dtype=dtype).reshape(shape)
        let arr = np
            .call_method(
                "frombuffer",
                (py_bytes,),
                Some(&pyo3::types::PyDict::new(py).tap(|d| {
                    let _ = d.set_item("dtype", self.dtype.numpy_dtype());
                })),
            )
            .map_err(BridgeError::from_pyo3)?;

        let shape_list: Vec<usize> = self.shape.clone();
        let reshaped = arr
            .call_method1("reshape", (shape_list,))
            .map_err(BridgeError::from_pyo3)?;

        Ok(reshaped.into())
    }
}

// ── Tiny helper trait for tap-style init (used above) ────────────────────────

#[cfg(feature = "cpython")]
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}

#[cfg(feature = "cpython")]
impl Tap for pyo3::types::PyDict {}
