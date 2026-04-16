// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! PostgreSQL driver via the synchronous `postgres` crate.
//!
//! # UniLang functions
//! | Function | Signature | Description |
//! |---|---|---|
//! | `pg_connect` | `pg_connect(url)` | Connect — e.g. `"host=localhost user=postgres password=secret dbname=mydb"` |
//! | `pg_query` | `pg_query(sql, params?)` | SELECT → `List[Dict]` |
//! | `pg_exec` | `pg_exec(sql, params?)` | INSERT/UPDATE/DELETE → rows affected |
//! | `pg_close` | `pg_close()` | Drop the connection |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

pub struct PostgresDriver {
    client: Arc<Mutex<Option<postgres::Client>>>,
}

impl PostgresDriver {
    pub fn new() -> Self {
        Self { client: Arc::new(Mutex::new(None)) }
    }
}

impl Default for PostgresDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for PostgresDriver {
    fn name(&self) -> &str { "postgres" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "PostgreSQL via synchronous postgres crate" }
    fn category(&self) -> DriverCategory { DriverCategory::SqlDatabase }
    fn exported_functions(&self) -> &'static [&'static str] {
        &["pg_connect", "pg_query", "pg_exec", "pg_close"]
    }

    fn register(&self, vm: &mut VM) {
        // pg_connect(url)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("pg_connect", move |args| {
                let url = str_arg(args, 0, "pg_connect(url)")?;
                let c = postgres::Client::connect(&url, postgres::NoTls)
                    .map_err(|e| RuntimeError::type_error(format!("pg_connect: {}", e)))?;
                *client.lock().unwrap() = Some(c);
                Ok(RuntimeValue::Bool(true))
            });
        }

        // pg_query(sql, params?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("pg_query", move |args| {
                let sql    = str_arg(args, 0, "pg_query(sql, params?)")?;
                let params = build_pg_params(args, 1);
                let mut guard = client.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("pg_query"))?;
                let dyn_params: Vec<&(dyn postgres::types::ToSql + Sync)> =
                    params.iter().map(|p| p as &(dyn postgres::types::ToSql + Sync)).collect();
                let rows = c.query(&sql as &str, &dyn_params[..])
                    .map_err(|e| RuntimeError::type_error(format!("pg_query: {}", e)))?;
                let result: Vec<RuntimeValue> = rows.iter().map(|row| {
                    let cols = row.columns();
                    let dict: Vec<(RuntimeValue, RuntimeValue)> = cols.iter().enumerate()
                        .map(|(i, col)| {
                            let key = RuntimeValue::String(col.name().to_string());
                            let val = pg_col_to_runtime(row, i);
                            (key, val)
                        })
                        .collect();
                    RuntimeValue::Dict(dict)
                }).collect();
                Ok(RuntimeValue::List(result))
            });
        }

        // pg_exec(sql, params?)
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("pg_exec", move |args| {
                let sql    = str_arg(args, 0, "pg_exec(sql, params?)")?;
                let params = build_pg_params(args, 1);
                let mut guard = client.lock().unwrap();
                let c = guard.as_mut().ok_or_else(|| no_conn("pg_exec"))?;
                let dyn_params: Vec<&(dyn postgres::types::ToSql + Sync)> =
                    params.iter().map(|p| p as &(dyn postgres::types::ToSql + Sync)).collect();
                let affected = c.execute(&sql as &str, &dyn_params[..])
                    .map_err(|e| RuntimeError::type_error(format!("pg_exec: {}", e)))?;
                Ok(RuntimeValue::Int(affected as i64))
            });
        }

        // pg_close()
        {
            let client = Arc::clone(&self.client);
            vm.register_builtin("pg_close", move |_args| {
                *client.lock().unwrap() = None;
                Ok(RuntimeValue::Null)
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call pg_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!("{}: expected string at position {}", sig, idx))),
    }
}

/// Convert UniLang list params to Vec<Box<dyn ToSql + Sync>>.
fn build_pg_params(args: &[RuntimeValue], idx: usize) -> Vec<Box<dyn postgres::types::ToSql + Sync>> {
    match args.get(idx) {
        Some(RuntimeValue::List(items)) => items.iter().map(to_pg_boxed).collect(),
        _ => vec![],
    }
}

fn to_pg_boxed(v: &RuntimeValue) -> Box<dyn postgres::types::ToSql + Sync> {
    match v {
        RuntimeValue::Null      => Box::new(Option::<String>::None),
        RuntimeValue::Bool(b)   => Box::new(*b),
        RuntimeValue::Int(n)    => Box::new(*n),
        RuntimeValue::Float(f)  => Box::new(*f),
        RuntimeValue::String(s) => Box::new(s.clone()),
        other                   => Box::new(format!("{}", other)),
    }
}

fn pg_col_to_runtime(row: &postgres::Row, idx: usize) -> RuntimeValue {
    // Try types in order of likelihood
    if let Ok(v) = row.try_get::<_, Option<i64>>(idx) {
        return v.map(RuntimeValue::Int).unwrap_or(RuntimeValue::Null);
    }
    if let Ok(v) = row.try_get::<_, Option<f64>>(idx) {
        return v.map(RuntimeValue::Float).unwrap_or(RuntimeValue::Null);
    }
    if let Ok(v) = row.try_get::<_, Option<bool>>(idx) {
        return v.map(RuntimeValue::Bool).unwrap_or(RuntimeValue::Null);
    }
    if let Ok(v) = row.try_get::<_, Option<String>>(idx) {
        return v.map(RuntimeValue::String).unwrap_or(RuntimeValue::Null);
    }
    RuntimeValue::Null
}
