// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! SQLite driver — embedded SQL database via `rusqlite`.
//!
//! # UniLang functions
//! | Function | Signature | Description |
//! |---|---|---|
//! | `db_connect` | `db_connect(path)` | Open (or create) a SQLite file; use `":memory:"` for an in-memory DB |
//! | `db_query` | `db_query(sql, params?)` | Run a SELECT — returns `List[Dict]` |
//! | `db_exec` | `db_exec(sql, params?)` | Run INSERT/UPDATE/DELETE — returns rows-affected count |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

pub struct SqliteDriver {
    conn: Arc<Mutex<Option<rusqlite::Connection>>>,
}

impl SqliteDriver {
    pub fn new() -> Self {
        Self { conn: Arc::new(Mutex::new(None)) }
    }
}

impl Default for SqliteDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for SqliteDriver {
    fn name(&self) -> &str { "sqlite" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Embedded SQLite database via rusqlite (bundled)" }
    fn category(&self) -> DriverCategory { DriverCategory::SqlDatabase }
    fn exported_functions(&self) -> &'static [&'static str] {
        &["db_connect", "db_query", "db_exec"]
    }

    fn register(&self, vm: &mut VM) {
        // ── db_connect ─────────────────────────────────────────────
        {
            let conn = Arc::clone(&self.conn);
            vm.register_builtin("db_connect", move |args| {
                let path = match args.first() {
                    Some(RuntimeValue::String(s)) => s.trim_start_matches("sqlite://").to_string(),
                    _ => return Err(RuntimeError::type_error("db_connect(path) requires a string")),
                };
                let c = if path == ":memory:" {
                    rusqlite::Connection::open_in_memory()
                } else {
                    rusqlite::Connection::open(&path)
                }.map_err(|e| RuntimeError::type_error(format!("db_connect: {}", e)))?;
                *conn.lock().unwrap() = Some(c);
                Ok(RuntimeValue::Bool(true))
            });
        }

        // ── db_query ──────────────────────────────────────────────
        {
            let conn = Arc::clone(&self.conn);
            vm.register_builtin("db_query", move |args| {
                let sql = match args.first() {
                    Some(RuntimeValue::String(s)) => s.clone(),
                    _ => return Err(RuntimeError::type_error("db_query(sql, params?)")),
                };
                let params = extract_params(args, 1);
                let guard = conn.lock().unwrap();
                let c = guard.as_ref()
                    .ok_or_else(|| RuntimeError::type_error("db_query: call db_connect() first"))?;
                let mut stmt = c.prepare(&sql)
                    .map_err(|e| RuntimeError::type_error(format!("db_query: {}", e)))?;
                let col_count = stmt.column_count();
                let col_names: Vec<String> = (0..col_count)
                    .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
                    .collect();
                let mut rows_val = stmt.query(
                    rusqlite::params_from_iter(params.iter()),
                ).map_err(|e| RuntimeError::type_error(format!("db_query: {}", e)))?;
                let mut result = Vec::new();
                while let Some(row) = rows_val.next()
                    .map_err(|e| RuntimeError::type_error(format!("db_query row: {}", e)))?
                {
                    let mut dict = Vec::new();
                    for (i, name) in col_names.iter().enumerate() {
                        let v = sql_col_to_runtime(row, i);
                        dict.push((RuntimeValue::String(name.clone()), v));
                    }
                    result.push(RuntimeValue::Dict(dict));
                }
                Ok(RuntimeValue::List(result))
            });
        }

        // ── db_exec ───────────────────────────────────────────────
        {
            let conn = Arc::clone(&self.conn);
            vm.register_builtin("db_exec", move |args| {
                let sql = match args.first() {
                    Some(RuntimeValue::String(s)) => s.clone(),
                    _ => return Err(RuntimeError::type_error("db_exec(sql, params?)")),
                };
                let params = extract_params(args, 1);
                let guard = conn.lock().unwrap();
                let c = guard.as_ref()
                    .ok_or_else(|| RuntimeError::type_error("db_exec: call db_connect() first"))?;
                let affected = c.execute(
                    &sql,
                    rusqlite::params_from_iter(params.iter()),
                ).map_err(|e| RuntimeError::type_error(format!("db_exec: {}", e)))?;
                Ok(RuntimeValue::Int(affected as i64))
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_params(args: &[RuntimeValue], idx: usize) -> Vec<rusqlite::types::Value> {
    match args.get(idx) {
        Some(RuntimeValue::List(items)) => items.iter().map(to_sql).collect(),
        _ => vec![],
    }
}

fn to_sql(v: &RuntimeValue) -> rusqlite::types::Value {
    match v {
        RuntimeValue::Int(n)    => rusqlite::types::Value::Integer(*n),
        RuntimeValue::Float(f)  => rusqlite::types::Value::Real(*f),
        RuntimeValue::Bool(b)   => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        RuntimeValue::String(s) => rusqlite::types::Value::Text(s.clone()),
        RuntimeValue::Null      => rusqlite::types::Value::Null,
        other                   => rusqlite::types::Value::Text(format!("{}", other)),
    }
}

fn sql_col_to_runtime(row: &rusqlite::Row<'_>, idx: usize) -> RuntimeValue {
    use rusqlite::types::ValueRef;
    match row.get_ref(idx).unwrap_or(ValueRef::Null) {
        ValueRef::Null         => RuntimeValue::Null,
        ValueRef::Integer(n)   => RuntimeValue::Int(n),
        ValueRef::Real(f)      => RuntimeValue::Float(f),
        ValueRef::Text(bytes)  => RuntimeValue::String(
            String::from_utf8_lossy(bytes).into_owned()
        ),
        ValueRef::Blob(b)      => RuntimeValue::String(format!("<blob {} bytes>", b.len())),
    }
}
