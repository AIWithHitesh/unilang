// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Java-backed thread pool bridge.
//!
//! [`JavaThreadPool`] wraps a `java.util.concurrent.ExecutorService` created via
//! `Executors.newFixedThreadPool(n)` and lets UniLang submit tasks that call
//! JVM static methods, collecting results asynchronously.

use crate::error::BridgeError;
#[cfg(feature = "jvm")]
use crate::jvm::JvmBridge;
use crate::types::BridgeValue;

/// A managed Java `ExecutorService` thread pool.
pub struct JavaThreadPool {
    #[cfg(feature = "jvm")]
    executor_handle: u64,
    #[cfg(feature = "jvm")]
    bridge: std::sync::Arc<JvmBridge>,
    #[cfg(feature = "jvm")]
    futures: std::sync::Arc<
        std::sync::Mutex<
            std::collections::HashMap<
                u64,
                std::thread::JoinHandle<Result<BridgeValue, BridgeError>>,
            >,
        >,
    >,
    #[cfg(feature = "jvm")]
    next_future: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

unsafe impl Send for JavaThreadPool {}
unsafe impl Sync for JavaThreadPool {}

impl JavaThreadPool {
    /// Create a fixed-size Java thread pool.
    ///
    /// Calls `java.util.concurrent.Executors.newFixedThreadPool(threads)` via JNI
    /// and stores the resulting `ExecutorService` handle.
    pub fn new(threads: usize) -> Result<Self, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            let bridge = std::sync::Arc::new(JvmBridge::new()?);
            let executor_handle = bridge.call_static(
                "java.util.concurrent.Executors",
                "newFixedThreadPool",
                &[BridgeValue::Int(threads as i64)],
            )?;
            let handle_int = match executor_handle {
                BridgeValue::Int(n) => n as u64,
                BridgeValue::JavaObject { handle, .. } => handle,
                _ => {
                    return Err(BridgeError::MarshalingError(
                        "unexpected return type from newFixedThreadPool".to_string(),
                    ))
                }
            };

            Ok(Self {
                executor_handle: handle_int,
                bridge,
                futures: std::sync::Arc::new(std::sync::Mutex::new(
                    std::collections::HashMap::new(),
                )),
                next_future: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            })
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = threads;
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Submit a static-method invocation to the pool and return a future handle.
    pub fn submit(
        &self,
        class: &str,
        method: &str,
        args: &[BridgeValue],
    ) -> Result<u64, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            let bridge = std::sync::Arc::clone(&self.bridge);
            let class = class.to_string();
            let method = method.to_string();
            let args: Vec<BridgeValue> = args.to_vec();

            let join_handle =
                std::thread::spawn(move || bridge.call_static(&class, &method, &args));

            let fid = self
                .next_future
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.futures
                .lock()
                .map_err(|_| BridgeError::MarshalingError("futures table mutex poisoned".into()))?
                .insert(fid, join_handle);

            Ok(fid)
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = (class, method, args);
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Block until the task identified by `future_handle` completes and return its value.
    pub fn await_result(&self, future_handle: u64) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            let handle = self
                .futures
                .lock()
                .map_err(|_| BridgeError::MarshalingError("futures table mutex poisoned".into()))?
                .remove(&future_handle)
                .ok_or_else(|| {
                    BridgeError::MarshalingError(format!("no future with handle {}", future_handle))
                })?;

            handle.join().map_err(|_| BridgeError::CrossVmException {
                source: "JVM".to_string(),
                message: "thread panicked during task execution".to_string(),
            })?
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = future_handle;
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Call `ExecutorService.shutdown()` on the underlying Java executor.
    pub fn shutdown(&self) -> Result<(), BridgeError> {
        #[cfg(feature = "jvm")]
        {
            self.bridge
                .call_instance(self.executor_handle, "shutdown", &[])?;
            Ok(())
        }
        #[cfg(not(feature = "jvm"))]
        {
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }
}
