// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! NATS driver — lightweight pub/sub messaging via the `nats` crate.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `nats_connect(url)` | Connect to NATS (e.g. `"nats://localhost:4222"`) |
//! | `nats_publish(subject, message)` | Publish a message |
//! | `nats_subscribe(subject)` | Subscribe to a subject |
//! | `nats_next_message(timeout_ms?)` | Get the next message on current subscription; returns String or Null |
//! | `nats_request(subject, message, timeout_ms?)` | Request-reply; returns String or Null on timeout |
//! | `nats_close()` | Close the connection |
//!
//! # Note
//! This is a stub implementation. The real implementation would use the `nats = "0.24"` crate
//! which provides a blocking synchronous API. To enable, add `nats = { version = "0.24", optional = true }`
//! to Cargo.toml and rebuild with `--features nats-driver`.

#[cfg(feature = "nats-driver")]
use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

#[cfg(feature = "nats-driver")]
#[allow(deprecated)]
struct NatsState {
    connection: nats::Connection,
    subscription: Option<nats::Subscription>,
}

pub struct NatsDriver {
    #[cfg(feature = "nats-driver")]
    state: Arc<Mutex<Option<NatsState>>>,
}

impl NatsDriver {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "nats-driver")]
            state: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for NatsDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl UniLangDriver for NatsDriver {
    fn name(&self) -> &str {
        "nats"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "NATS lightweight pub/sub messaging"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Queue
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "nats_connect",
            "nats_publish",
            "nats_subscribe",
            "nats_next_message",
            "nats_request",
            "nats_close",
        ]
    }

    #[cfg(not(feature = "nats-driver"))]
    fn register(&self, vm: &mut VM) {
        // Stub implementations — return descriptive errors at runtime
        let stub = |name: &'static str| {
            move |_args: &[RuntimeValue]| -> Result<RuntimeValue, RuntimeError> {
                Err(RuntimeError::type_error(format!(
                    "{}: nats driver requires the 'nats-driver' feature and a running NATS server",
                    name
                )))
            }
        };
        vm.register_builtin("nats_connect", stub("nats_connect"));
        vm.register_builtin("nats_publish", stub("nats_publish"));
        vm.register_builtin("nats_subscribe", stub("nats_subscribe"));
        vm.register_builtin("nats_next_message", stub("nats_next_message"));
        vm.register_builtin("nats_request", stub("nats_request"));
        vm.register_builtin("nats_close", stub("nats_close"));
    }

    #[cfg(feature = "nats-driver")]
    #[allow(deprecated)]
    fn register(&self, vm: &mut VM) {
        use std::time::Duration;

        macro_rules! arc {
            () => {
                Arc::clone(&self.state)
            };
        }

        // nats_connect(url)
        {
            let state = arc!();
            vm.register_builtin("nats_connect", move |args| {
                let url = str_arg(args, 0, "nats_connect(url)")?;
                let nc = nats::connect(&url)
                    .map_err(|e| RuntimeError::type_error(format!("nats_connect: {}", e)))?;
                *state.lock().unwrap() = Some(NatsState {
                    connection: nc,
                    subscription: None,
                });
                Ok(RuntimeValue::Bool(true))
            });
        }

        // nats_publish(subject, message)
        {
            let state = arc!();
            vm.register_builtin("nats_publish", move |args| {
                let subject = str_arg(args, 0, "nats_publish(subject, message)")?;
                let message = str_arg(args, 1, "nats_publish(subject, message)")?;
                let guard = state.lock().unwrap();
                let s = guard.as_ref().ok_or_else(|| no_conn("nats_publish"))?;
                s.connection
                    .publish(&subject, message.as_bytes())
                    .map_err(|e| RuntimeError::type_error(format!("nats_publish: {}", e)))?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // nats_subscribe(subject)
        {
            let state = arc!();
            vm.register_builtin("nats_subscribe", move |args| {
                let subject = str_arg(args, 0, "nats_subscribe(subject)")?;
                let mut guard = state.lock().unwrap();
                let s = guard.as_mut().ok_or_else(|| no_conn("nats_subscribe"))?;
                let sub = s
                    .connection
                    .subscribe(&subject)
                    .map_err(|e| RuntimeError::type_error(format!("nats_subscribe: {}", e)))?;
                s.subscription = Some(sub);
                Ok(RuntimeValue::Bool(true))
            });
        }

        // nats_next_message(timeout_ms?)
        {
            let state = arc!();
            vm.register_builtin("nats_next_message", move |args| {
                let timeout_ms = int_arg(args, 0).unwrap_or(5000) as u64;
                let guard = state.lock().unwrap();
                let s = guard.as_ref().ok_or_else(|| no_conn("nats_next_message"))?;
                let sub = s.subscription.as_ref().ok_or_else(|| {
                    RuntimeError::type_error(
                        "nats_next_message: call nats_subscribe() first".to_string(),
                    )
                })?;
                match sub.next_timeout(Duration::from_millis(timeout_ms)) {
                    Ok(msg) => {
                        let body = String::from_utf8_lossy(&msg.data).to_string();
                        Ok(RuntimeValue::String(body))
                    }
                    Err(_) => Ok(RuntimeValue::Null),
                }
            });
        }

        // nats_request(subject, message, timeout_ms?)
        {
            let state = arc!();
            vm.register_builtin("nats_request", move |args| {
                let subject = str_arg(args, 0, "nats_request(subject, message, timeout_ms?)")?;
                let message = str_arg(args, 1, "nats_request(subject, message, timeout_ms?)")?;
                let timeout_ms = int_arg(args, 2).unwrap_or(5000) as u64;
                let guard = state.lock().unwrap();
                let s = guard.as_ref().ok_or_else(|| no_conn("nats_request"))?;
                match s.connection.request_timeout(
                    &subject,
                    message.as_bytes(),
                    Duration::from_millis(timeout_ms),
                ) {
                    Ok(msg) => {
                        let body = String::from_utf8_lossy(&msg.data).to_string();
                        Ok(RuntimeValue::String(body))
                    }
                    Err(_) => Ok(RuntimeValue::Null),
                }
            });
        }

        // nats_close()
        {
            let state = arc!();
            vm.register_builtin("nats_close", move |_args| {
                let mut guard = state.lock().unwrap();
                if let Some(s) = guard.take() {
                    drop(s.subscription);
                    s.connection.close();
                }
                Ok(RuntimeValue::Bool(true))
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call nats_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        Some(other) => Ok(format!("{}", other)),
        None => Err(RuntimeError::type_error(format!(
            "{}: missing arg at position {}",
            sig, idx
        ))),
    }
}

fn int_arg(args: &[RuntimeValue], idx: usize) -> Option<i64> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n)) => Some(*n),
        Some(RuntimeValue::Float(f)) => Some(*f as i64),
        _ => None,
    }
}
