// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! RabbitMQ driver — AMQP message broker via the `lapin` crate.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `rabbitmq_connect(url)` | Connect to RabbitMQ (e.g. `"amqp://user:pass@localhost:5672/vhost"`) |
//! | `rabbitmq_declare_queue(queue_name)` | Declare a durable queue |
//! | `rabbitmq_publish(exchange, routing_key, message)` | Publish a message |
//! | `rabbitmq_consume_one(queue_name, timeout_ms?)` | Get one message (auto-ack), returns String or Null |
//! | `rabbitmq_close()` | Close the connection |
//!
//! # Note
//! This is a stub implementation. The real implementation would use the `lapin` crate
//! (an async AMQP client) with a `tokio::runtime::Runtime` for blocking use.
//! To enable: add `lapin = { version = "2", optional = true }` and
//! `tokio = { version = "1", features = ["rt", "rt-multi-thread"], optional = true }` to Cargo.toml,
//! then rebuild with `--features rabbitmq`.

#[cfg(feature = "rabbitmq")]
use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

#[cfg(feature = "rabbitmq")]
use lapin::{Channel, Connection, ConnectionProperties};

#[cfg(feature = "rabbitmq")]
struct RabbitMqState {
    connection: Connection,
    channel: Channel,
}

pub struct RabbitMqDriver {
    #[cfg(feature = "rabbitmq")]
    state: Arc<Mutex<Option<RabbitMqState>>>,
}

impl RabbitMqDriver {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rabbitmq")]
            state: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for RabbitMqDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl UniLangDriver for RabbitMqDriver {
    fn name(&self) -> &str {
        "rabbitmq"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "RabbitMQ AMQP message broker (publish/consume)"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Queue
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "rabbitmq_connect",
            "rabbitmq_declare_queue",
            "rabbitmq_publish",
            "rabbitmq_consume_one",
            "rabbitmq_close",
        ]
    }

    #[cfg(not(feature = "rabbitmq"))]
    fn register(&self, vm: &mut VM) {
        // Stub implementations — return descriptive errors at runtime
        let stub = |name: &'static str| {
            move |_args: &[RuntimeValue]| -> Result<RuntimeValue, RuntimeError> {
                Err(RuntimeError::type_error(format!(
                    "{}: rabbitmq driver requires the 'rabbitmq' feature and a running RabbitMQ server",
                    name
                )))
            }
        };
        vm.register_builtin("rabbitmq_connect", stub("rabbitmq_connect"));
        vm.register_builtin("rabbitmq_declare_queue", stub("rabbitmq_declare_queue"));
        vm.register_builtin("rabbitmq_publish", stub("rabbitmq_publish"));
        vm.register_builtin("rabbitmq_consume_one", stub("rabbitmq_consume_one"));
        vm.register_builtin("rabbitmq_close", stub("rabbitmq_close"));
    }

    #[cfg(feature = "rabbitmq")]
    fn register(&self, vm: &mut VM) {
        use lapin::{options::*, types::FieldTable, BasicProperties};
        use tokio::runtime::Runtime;

        macro_rules! arc {
            () => {
                Arc::clone(&self.state)
            };
        }

        // rabbitmq_connect(url)
        {
            let state = arc!();
            vm.register_builtin("rabbitmq_connect", move |args| {
                let url = str_arg(args, 0, "rabbitmq_connect(url)")?;
                let rt = Runtime::new().map_err(|e| {
                    RuntimeError::type_error(format!("rabbitmq_connect: tokio runtime: {}", e))
                })?;
                let (conn, chan) = rt.block_on(async {
                    let conn = Connection::connect(&url, ConnectionProperties::default())
                        .await
                        .map_err(|e| {
                            RuntimeError::type_error(format!("rabbitmq_connect: {}", e))
                        })?;
                    let chan = conn.create_channel().await.map_err(|e| {
                        RuntimeError::type_error(format!("rabbitmq_connect channel: {}", e))
                    })?;
                    Ok::<(Connection, Channel), RuntimeError>((conn, chan))
                })?;
                *state.lock().unwrap() = Some(RabbitMqState {
                    connection: conn,
                    channel: chan,
                });
                Ok(RuntimeValue::Bool(true))
            });
        }

        // rabbitmq_declare_queue(queue_name)
        {
            let state = arc!();
            vm.register_builtin("rabbitmq_declare_queue", move |args| {
                let queue = str_arg(args, 0, "rabbitmq_declare_queue(queue_name)")?;
                let rt = Runtime::new().map_err(|e| {
                    RuntimeError::type_error(format!("rabbitmq_declare_queue: {}", e))
                })?;
                let guard = state.lock().unwrap();
                let s = guard
                    .as_ref()
                    .ok_or_else(|| no_conn("rabbitmq_declare_queue"))?;
                rt.block_on(async {
                    s.channel
                        .queue_declare(
                            &queue,
                            QueueDeclareOptions {
                                durable: true,
                                ..Default::default()
                            },
                            FieldTable::default(),
                        )
                        .await
                        .map_err(|e| {
                            RuntimeError::type_error(format!("rabbitmq_declare_queue: {}", e))
                        })
                })?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // rabbitmq_publish(exchange, routing_key, message)
        {
            let state = arc!();
            vm.register_builtin("rabbitmq_publish", move |args| {
                let exchange =
                    str_arg(args, 0, "rabbitmq_publish(exchange, routing_key, message)")?;
                let routing_key =
                    str_arg(args, 1, "rabbitmq_publish(exchange, routing_key, message)")?;
                let message = str_arg(args, 2, "rabbitmq_publish(exchange, routing_key, message)")?;
                let rt = Runtime::new()
                    .map_err(|e| RuntimeError::type_error(format!("rabbitmq_publish: {}", e)))?;
                let guard = state.lock().unwrap();
                let s = guard.as_ref().ok_or_else(|| no_conn("rabbitmq_publish"))?;
                rt.block_on(async {
                    s.channel
                        .basic_publish(
                            &exchange,
                            &routing_key,
                            BasicPublishOptions::default(),
                            message.as_bytes(),
                            BasicProperties::default(),
                        )
                        .await
                        .map_err(|e| RuntimeError::type_error(format!("rabbitmq_publish: {}", e)))
                })?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // rabbitmq_consume_one(queue_name, timeout_ms?)
        {
            let state = arc!();
            vm.register_builtin("rabbitmq_consume_one", move |args| {
                let queue = str_arg(args, 0, "rabbitmq_consume_one(queue_name, timeout_ms?)")?;
                let timeout_ms = int_arg(args, 1).unwrap_or(5000) as u64;
                let rt = Runtime::new().map_err(|e| {
                    RuntimeError::type_error(format!("rabbitmq_consume_one: {}", e))
                })?;
                let guard = state.lock().unwrap();
                let s = guard
                    .as_ref()
                    .ok_or_else(|| no_conn("rabbitmq_consume_one"))?;
                let result = rt.block_on(async {
                    use futures_lite::stream::StreamExt;
                    use tokio::time::{timeout, Duration};
                    let mut consumer = s
                        .channel
                        .basic_consume(
                            &queue,
                            "unilang_consumer",
                            BasicConsumeOptions::default(),
                            FieldTable::default(),
                        )
                        .await
                        .map_err(|e| {
                            RuntimeError::type_error(format!("rabbitmq_consume_one: {}", e))
                        })?;
                    let maybe = timeout(Duration::from_millis(timeout_ms), consumer.next()).await;
                    match maybe {
                        Ok(Some(Ok(delivery))) => {
                            delivery.ack(BasicAckOptions::default()).await.ok();
                            let body = String::from_utf8_lossy(&delivery.data).to_string();
                            Ok::<RuntimeValue, RuntimeError>(RuntimeValue::String(body))
                        }
                        _ => Ok(RuntimeValue::Null),
                    }
                })?;
                Ok(result)
            });
        }

        // rabbitmq_close()
        {
            let state = arc!();
            vm.register_builtin("rabbitmq_close", move |_args| {
                let mut guard = state.lock().unwrap();
                if let Some(s) = guard.take() {
                    let rt = Runtime::new().ok();
                    if let Some(rt) = rt {
                        rt.block_on(async {
                            s.connection.close(0, "bye").await.ok();
                        });
                    }
                }
                Ok(RuntimeValue::Bool(true))
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call rabbitmq_connect() first", func))
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
