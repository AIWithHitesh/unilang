// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Kafka driver — in-process event log (default) with hooks for real Kafka.
//!
//! The default implementation is a **in-memory event log** so that UniLang
//! scripts can use Kafka-style publish/consume patterns without requiring a
//! running broker. This is ideal for development and testing.
//!
//! For production use with a real Kafka cluster, replace this crate with
//! `unilang-driver-kafka-rdkafka` (wraps `rdkafka`).
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `kafka_connect(brokers)` | Set broker URL(s) — no-op for in-memory mode |
//! | `kafka_produce(topic, key, payload)` | Publish a message |
//! | `kafka_events(topic?)` | Return all events (optionally filter by topic) |
//! | `kafka_clear(topic?)` | Clear the event log |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

/// One Kafka event — (topic, key, payload string).
type Event = (String, String, String);

pub struct KafkaDriver {
    log: Arc<Mutex<Vec<Event>>>,
}

impl KafkaDriver {
    pub fn new() -> Self {
        Self { log: Arc::new(Mutex::new(Vec::new())) }
    }
}

impl Default for KafkaDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for KafkaDriver {
    fn name(&self) -> &str { "kafka" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str {
        "In-memory Kafka event log (dev/test). Swap for unilang-driver-kafka-rdkafka in production."
    }
    fn category(&self) -> DriverCategory { DriverCategory::Queue }
    fn exported_functions(&self) -> &'static [&'static str] {
        &["kafka_connect", "kafka_produce", "kafka_events", "kafka_clear"]
    }

    fn register(&self, vm: &mut VM) {
        // kafka_connect — no-op for in-memory, stores broker string for future use
        vm.register_builtin("kafka_connect", |_args| {
            Ok(RuntimeValue::Bool(true))
        });

        // kafka_produce(topic, key, payload)
        {
            let log = Arc::clone(&self.log);
            vm.register_builtin("kafka_produce", move |args| {
                let topic   = str_arg(args, 0, "kafka_produce(topic, key, payload)")?;
                let key     = str_arg(args, 1, "kafka_produce(topic, key, payload)")?;
                let payload = str_arg(args, 2, "kafka_produce(topic, key, payload)")?;
                println!("[KAFKA] topic={} key={} payload={}", topic, key, payload);
                log.lock().unwrap().push((topic, key, payload));
                Ok(RuntimeValue::Null)
            });
        }

        // kafka_events(topic?) — returns List[Dict{topic,key,payload}]
        {
            let log = Arc::clone(&self.log);
            vm.register_builtin("kafka_events", move |args| {
                let filter = match args.first() {
                    Some(RuntimeValue::String(t)) => Some(t.clone()),
                    _ => None,
                };
                let guard = log.lock().unwrap();
                let events: Vec<RuntimeValue> = guard.iter()
                    .filter(|(t, _, _)| filter.as_ref().map_or(true, |f| f == t))
                    .map(|(t, k, p)| RuntimeValue::Dict(vec![
                        (RuntimeValue::String("topic".into()),   RuntimeValue::String(t.clone())),
                        (RuntimeValue::String("key".into()),     RuntimeValue::String(k.clone())),
                        (RuntimeValue::String("payload".into()), RuntimeValue::String(p.clone())),
                    ]))
                    .collect();
                Ok(RuntimeValue::List(events))
            });
        }

        // kafka_clear(topic?) — clear all or specific-topic events
        {
            let log = Arc::clone(&self.log);
            vm.register_builtin("kafka_clear", move |args| {
                let filter = match args.first() {
                    Some(RuntimeValue::String(t)) => Some(t.clone()),
                    _ => None,
                };
                let mut guard = log.lock().unwrap();
                match filter {
                    Some(f) => guard.retain(|(t, _, _)| t != &f),
                    None    => guard.clear(),
                }
                Ok(RuntimeValue::Null)
            });
        }
    }
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        Some(other) => Ok(format!("{}", other)),
        None => Err(RuntimeError::type_error(format!("{}: missing arg at position {}", sig, idx))),
    }
}
