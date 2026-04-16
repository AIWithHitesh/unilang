// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Memcached driver via the `memcache` crate.
//!
//! # UniLang functions
//! | Function | Signature | Description |
//! |---|---|---|
//! | `memcached_connect` | `memcached_connect(url)` | Connect — e.g. `"memcache://127.0.0.1:11211"` |
//! | `memcached_get` | `memcached_get(key)` | Get value → String or `null` |
//! | `memcached_set` | `memcached_set(key, value, ttl?)` | Set with optional TTL (seconds) |
//! | `memcached_delete` | `memcached_delete(key)` | Delete a key |
//! | `memcached_add` | `memcached_add(key, value, ttl?)` | Set only if key does not exist |
//! | `memcached_replace` | `memcached_replace(key, value, ttl?)` | Set only if key exists |
//! | `memcached_incr` | `memcached_incr(key, delta?)` | Increment counter |
//! | `memcached_decr` | `memcached_decr(key, delta?)` | Decrement counter |
//! | `memcached_flush` | `memcached_flush()` | Flush all keys |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

pub struct MemcachedDriver {
    client: Arc<Mutex<Option<memcache::Client>>>,
}

impl MemcachedDriver {
    pub fn new() -> Self {
        Self { client: Arc::new(Mutex::new(None)) }
    }
}

impl Default for MemcachedDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for MemcachedDriver {
    fn name(&self) -> &str { "memcached" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Memcached via pure-Rust memcache crate" }
    fn category(&self) -> DriverCategory { DriverCategory::Cache }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "memcached_connect",
            "memcached_get", "memcached_set", "memcached_delete",
            "memcached_add", "memcached_replace",
            "memcached_incr", "memcached_decr",
            "memcached_flush",
        ]
    }

    fn register(&self, vm: &mut VM) {
        // memcached_connect(url)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_connect", move |args| {
                let url = str_arg(args, 0, "memcached_connect(url)")?;
                let c = memcache::Client::connect(&url as &str)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_connect: {}", e)))?;
                *client.lock().unwrap() = Some(c);
                Ok(RuntimeValue::Bool(true))
            });
        }

        // memcached_get(key)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_get", move |args| {
                let key = str_arg(args, 0, "memcached_get(key)")?;
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_get"))?;
                let result: Option<String> = c.get(&key)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_get: {}", e)))?;
                Ok(match result {
                    Some(s) => RuntimeValue::String(s),
                    None => RuntimeValue::Null,
                })
            });
        }

        // memcached_set(key, value, ttl?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_set", move |args| {
                let key = str_arg(args, 0, "memcached_set(key, value, ttl?)")?;
                let val = val_to_string(args.get(1))?;
                let ttl = u32_arg(args, 2).unwrap_or(0);
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_set"))?;
                c.set(&key, val.as_str(), ttl)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_set: {}", e)))?;
                Ok(RuntimeValue::Null)
            });
        }

        // memcached_delete(key)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_delete", move |args| {
                let key = str_arg(args, 0, "memcached_delete(key)")?;
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_delete"))?;
                c.delete(&key)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_delete: {}", e)))?;
                Ok(RuntimeValue::Null)
            });
        }

        // memcached_add(key, value, ttl?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_add", move |args| {
                let key = str_arg(args, 0, "memcached_add(key, value, ttl?)")?;
                let val = val_to_string(args.get(1))?;
                let ttl = u32_arg(args, 2).unwrap_or(0);
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_add"))?;
                c.add(&key, val.as_str(), ttl)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_add: {}", e)))?;
                Ok(RuntimeValue::Null)
            });
        }

        // memcached_replace(key, value, ttl?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_replace", move |args| {
                let key = str_arg(args, 0, "memcached_replace(key, value, ttl?)")?;
                let val = val_to_string(args.get(1))?;
                let ttl = u32_arg(args, 2).unwrap_or(0);
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_replace"))?;
                c.replace(&key, val.as_str(), ttl)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_replace: {}", e)))?;
                Ok(RuntimeValue::Null)
            });
        }

        // memcached_incr(key, delta?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_incr", move |args| {
                let key   = str_arg(args, 0, "memcached_incr(key, delta?)")?;
                let delta = u64_arg(args, 1).unwrap_or(1);
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_incr"))?;
                let new_val = c.increment(&key, delta)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_incr: {}", e)))?;
                Ok(RuntimeValue::Int(new_val as i64))
            });
        }

        // memcached_decr(key, delta?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_decr", move |args| {
                let key   = str_arg(args, 0, "memcached_decr(key, delta?)")?;
                let delta = u64_arg(args, 1).unwrap_or(1);
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_decr"))?;
                let new_val = c.decrement(&key, delta)
                    .map_err(|e| RuntimeError::type_error(format!("memcached_decr: {}", e)))?;
                Ok(RuntimeValue::Int(new_val as i64))
            });
        }

        // memcached_flush()
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("memcached_flush", move |_args| {
                let guard = client.lock().unwrap();
                let c = guard.as_ref().ok_or_else(|| no_conn("memcached_flush"))?;
                c.flush()
                    .map_err(|e| RuntimeError::type_error(format!("memcached_flush: {}", e)))?;
                Ok(RuntimeValue::Null)
            });
        }
    }
}

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call memcached_connect() first", func))
}
fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!("{}: expected string at position {}", sig, idx))),
    }
}
fn val_to_string(v: Option<&RuntimeValue>) -> Result<String, RuntimeError> {
    match v {
        Some(x) => Ok(format!("{}", x)),
        None => Err(RuntimeError::type_error("missing value argument")),
    }
}
fn u32_arg(args: &[RuntimeValue], idx: usize) -> Option<u32> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n))   => Some(*n as u32),
        Some(RuntimeValue::Float(f)) => Some(*f as u32),
        _ => None,
    }
}
fn u64_arg(args: &[RuntimeValue], idx: usize) -> Option<u64> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n))   => Some(*n as u64),
        Some(RuntimeValue::Float(f)) => Some(*f as u64),
        _ => None,
    }
}
