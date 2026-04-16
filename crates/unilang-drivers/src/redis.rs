// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Redis driver — remote in-memory data store via the `redis` crate.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `redis_connect(url)` | Connect to Redis (e.g. `"redis://127.0.0.1:6379"`) |
//! | `redis_get(key)` | Get a string value; returns `null` if missing |
//! | `redis_set(key, value)` | Set a key with no expiry |
//! | `redis_setex(key, seconds, value)` | Set a key with TTL in seconds |
//! | `redis_del(key)` | Delete a key |
//! | `redis_exists(key)` | Returns `true` if key exists |
//! | `redis_incr(key)` | Increment integer value by 1; returns new value |
//! | `redis_hset(key, field, value)` | Set hash field |
//! | `redis_hget(key, field)` | Get hash field |
//! | `redis_hgetall(key)` | Get all fields as `Dict` |
//! | `redis_hdel(key, field)` | Delete hash field |
//! | `redis_expire(key, seconds)` | Set TTL on existing key |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

pub struct RedisDriver {
    conn: Arc<Mutex<Option<redis::Connection>>>,
}

impl RedisDriver {
    pub fn new() -> Self {
        Self { conn: Arc::new(Mutex::new(None)) }
    }
}

impl Default for RedisDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for RedisDriver {
    fn name(&self) -> &str { "redis" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Redis in-memory data store (strings, hashes, TTL)" }
    fn category(&self) -> DriverCategory { DriverCategory::Cache }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "redis_connect",
            "redis_get", "redis_set", "redis_setex",
            "redis_del", "redis_exists", "redis_incr",
            "redis_hset", "redis_hget", "redis_hgetall", "redis_hdel",
            "redis_expire",
        ]
    }

    fn register(&self, vm: &mut VM) {
        macro_rules! arc {
            () => { Arc::clone(&self.conn) };
        }

        // redis_connect
        {
            let conn = arc!();
            vm.register_builtin("redis_connect", move |args| {
                let url = str_arg(args, 0, "redis_connect(url)")?;
                let client = redis::Client::open(url.as_str())
                    .map_err(|e| RuntimeError::type_error(format!("redis_connect: {}", e)))?;
                let c = client.get_connection()
                    .map_err(|e| RuntimeError::type_error(format!("redis_connect: {}", e)))?;
                *conn.lock().unwrap() = Some(c);
                Ok(RuntimeValue::Bool(true))
            });
        }

        // redis_get
        {
            let conn = arc!();
            vm.register_builtin("redis_get", move |args| {
                let key = str_arg(args, 0, "redis_get(key)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_get"))?;
                let result: Option<String> = redis::cmd("GET").arg(&key).query(c)
                    .unwrap_or(None);
                Ok(match result {
                    Some(s) => RuntimeValue::String(s),
                    None => RuntimeValue::Null,
                })
            });
        }

        // redis_set
        {
            let conn = arc!();
            vm.register_builtin("redis_set", move |args| {
                let key = str_arg(args, 0, "redis_set(key, value)")?;
                let val = val_to_string(args.get(1))?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_set"))?;
                redis::cmd("SET").arg(&key).arg(&val).execute(c);
                Ok(RuntimeValue::Null)
            });
        }

        // redis_setex(key, seconds, value)
        {
            let conn = arc!();
            vm.register_builtin("redis_setex", move |args| {
                let key  = str_arg(args, 0, "redis_setex(key, seconds, value)")?;
                let secs = int_arg(args, 1).unwrap_or(3600);
                let val  = val_to_string(args.get(2))?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_setex"))?;
                redis::cmd("SETEX").arg(&key).arg(secs).arg(&val).execute(c);
                Ok(RuntimeValue::Null)
            });
        }

        // redis_del
        {
            let conn = arc!();
            vm.register_builtin("redis_del", move |args| {
                let key = str_arg(args, 0, "redis_del(key)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_del"))?;
                redis::cmd("DEL").arg(&key).execute(c);
                Ok(RuntimeValue::Null)
            });
        }

        // redis_exists
        {
            let conn = arc!();
            vm.register_builtin("redis_exists", move |args| {
                let key = str_arg(args, 0, "redis_exists(key)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_exists"))?;
                let n: i64 = redis::cmd("EXISTS").arg(&key).query(c).unwrap_or(0);
                Ok(RuntimeValue::Bool(n > 0))
            });
        }

        // redis_incr
        {
            let conn = arc!();
            vm.register_builtin("redis_incr", move |args| {
                let key = str_arg(args, 0, "redis_incr(key)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_incr"))?;
                let n: i64 = redis::cmd("INCR").arg(&key).query(c).unwrap_or(0);
                Ok(RuntimeValue::Int(n))
            });
        }

        // redis_hset
        {
            let conn = arc!();
            vm.register_builtin("redis_hset", move |args| {
                let key   = str_arg(args, 0, "redis_hset(key, field, value)")?;
                let field = str_arg(args, 1, "redis_hset(key, field, value)")?;
                let val   = val_to_string(args.get(2))?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_hset"))?;
                redis::cmd("HSET").arg(&key).arg(&field).arg(&val).execute(c);
                Ok(RuntimeValue::Null)
            });
        }

        // redis_hget
        {
            let conn = arc!();
            vm.register_builtin("redis_hget", move |args| {
                let key   = str_arg(args, 0, "redis_hget(key, field)")?;
                let field = str_arg(args, 1, "redis_hget(key, field)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_hget"))?;
                let result: Option<String> = redis::cmd("HGET").arg(&key).arg(&field).query(c).unwrap_or(None);
                Ok(match result {
                    Some(s) => RuntimeValue::String(s),
                    None => RuntimeValue::Null,
                })
            });
        }

        // redis_hgetall
        {
            let conn = arc!();
            vm.register_builtin("redis_hgetall", move |args| {
                let key = str_arg(args, 0, "redis_hgetall(key)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_hgetall"))?;
                let pairs: Vec<String> = redis::cmd("HGETALL").arg(&key).query(c).unwrap_or_default();
                let dict: Vec<(RuntimeValue, RuntimeValue)> = pairs
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            Some((
                                RuntimeValue::String(chunk[0].clone()),
                                RuntimeValue::String(chunk[1].clone()),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(RuntimeValue::Dict(dict))
            });
        }

        // redis_hdel
        {
            let conn = arc!();
            vm.register_builtin("redis_hdel", move |args| {
                let key   = str_arg(args, 0, "redis_hdel(key, field)")?;
                let field = str_arg(args, 1, "redis_hdel(key, field)")?;
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_hdel"))?;
                redis::cmd("HDEL").arg(&key).arg(&field).execute(c);
                Ok(RuntimeValue::Null)
            });
        }

        // redis_expire
        {
            let conn = arc!();
            vm.register_builtin("redis_expire", move |args| {
                let key  = str_arg(args, 0, "redis_expire(key, seconds)")?;
                let secs = int_arg(args, 1).unwrap_or(3600);
                let mut guard = conn.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("redis_expire"))?;
                redis::cmd("EXPIRE").arg(&key).arg(secs).execute(c);
                Ok(RuntimeValue::Null)
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call redis_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!("{} requires string at position {}", sig, idx))),
    }
}

fn int_arg(args: &[RuntimeValue], idx: usize) -> Option<i64> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n))   => Some(*n),
        Some(RuntimeValue::Float(f)) => Some(*f as i64),
        _ => None,
    }
}

fn val_to_string(v: Option<&RuntimeValue>) -> Result<String, RuntimeError> {
    match v {
        Some(x) => Ok(format!("{}", x)),
        None => Err(RuntimeError::type_error("missing value argument")),
    }
}
