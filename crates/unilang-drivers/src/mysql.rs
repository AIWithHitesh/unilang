// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! MySQL / MariaDB driver via the pure-Rust `mysql` crate.
//!
//! # UniLang functions
//! | Function | Signature | Description |
//! |---|---|---|
//! | `mysql_connect` | `mysql_connect(url)` | Connect — e.g. `"mysql://user:pass@host:3306/db"` |
//! | `mysql_query` | `mysql_query(sql, params?)` | SELECT → `List[Dict]` |
//! | `mysql_exec` | `mysql_exec(sql, params?)` | INSERT/UPDATE/DELETE → rows affected |
//! | `mysql_close` | `mysql_close()` | Release the connection |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

pub struct MySqlDriver {
    pool: Arc<Mutex<Option<mysql::Pool>>>,
}

impl MySqlDriver {
    pub fn new() -> Self {
        Self { pool: Arc::new(Mutex::new(None)) }
    }
}

impl Default for MySqlDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for MySqlDriver {
    fn name(&self) -> &str { "mysql" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "MySQL / MariaDB via pure-Rust mysql crate" }
    fn category(&self) -> DriverCategory { DriverCategory::SqlDatabase }
    fn exported_functions(&self) -> &'static [&'static str] {
        &["mysql_connect", "mysql_query", "mysql_exec", "mysql_close"]
    }

    fn register(&self, vm: &mut VM) {
        // mysql_connect(url)
        {
            let pool = Arc::clone(&self.pool);
            vm.register_builtin("mysql_connect", move |args| {
                let url = str_arg(args, 0, "mysql_connect(url)")?;
                let opts = mysql::Opts::from_url(&url)
                    .map_err(|e| RuntimeError::type_error(format!("mysql_connect: {}", e)))?;
                let p = mysql::Pool::new(opts)
                    .map_err(|e| RuntimeError::type_error(format!("mysql_connect: {}", e)))?;
                *pool.lock().unwrap() = Some(p);
                Ok(RuntimeValue::Bool(true))
            });
        }

        // mysql_query(sql, params?)
        {
            let pool = Arc::clone(&self.pool);
            vm.register_builtin("mysql_query", move |args| {
                use mysql::prelude::Queryable;
                let sql    = str_arg(args, 0, "mysql_query(sql, params?)")?;
                let params = build_mysql_params(args, 1);
                let guard  = pool.lock().unwrap();
                let p = guard.as_ref().ok_or_else(|| no_conn("mysql_query"))?;
                let mut conn = p.get_conn()
                    .map_err(|e| RuntimeError::type_error(format!("mysql_query: {}", e)))?;
                let rows: Vec<mysql::Row> = conn.exec(&sql, params)
                    .map_err(|e| RuntimeError::type_error(format!("mysql_query: {}", e)))?;
                let result: Vec<RuntimeValue> = rows.iter().map(|row| {
                    let columns = row.columns_ref();
                    let dict: Vec<(RuntimeValue, RuntimeValue)> = columns.iter().enumerate()
                        .map(|(i, col)| {
                            let key = RuntimeValue::String(col.name_str().into_owned());
                            let val = mysql_val_to_runtime(&row[i]);
                            (key, val)
                        })
                        .collect();
                    RuntimeValue::Dict(dict)
                }).collect();
                Ok(RuntimeValue::List(result))
            });
        }

        // mysql_exec(sql, params?)
        {
            let pool = Arc::clone(&self.pool);
            vm.register_builtin("mysql_exec", move |args| {
                use mysql::prelude::Queryable;
                let sql    = str_arg(args, 0, "mysql_exec(sql, params?)")?;
                let params = build_mysql_params(args, 1);
                let guard  = pool.lock().unwrap();
                let p = guard.as_ref().ok_or_else(|| no_conn("mysql_exec"))?;
                let mut conn = p.get_conn()
                    .map_err(|e| RuntimeError::type_error(format!("mysql_exec: {}", e)))?;
                conn.exec_drop(&sql, params)
                    .map_err(|e| RuntimeError::type_error(format!("mysql_exec: {}", e)))?;
                Ok(RuntimeValue::Int(conn.affected_rows() as i64))
            });
        }

        // mysql_close()
        {
            let pool = Arc::clone(&self.pool);
            vm.register_builtin("mysql_close", move |_args| {
                *pool.lock().unwrap() = None;
                Ok(RuntimeValue::Null)
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call mysql_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!("{}: expected string at position {}", sig, idx))),
    }
}

fn build_mysql_params(args: &[RuntimeValue], idx: usize) -> mysql::Params {
    match args.get(idx) {
        Some(RuntimeValue::List(items)) => {
            let vals: Vec<mysql::Value> = items.iter().map(to_mysql_value).collect();
            mysql::Params::Positional(vals)
        }
        _ => mysql::Params::Empty,
    }
}

fn to_mysql_value(v: &RuntimeValue) -> mysql::Value {
    match v {
        RuntimeValue::Null       => mysql::Value::NULL,
        RuntimeValue::Bool(b)    => mysql::Value::Int(if *b { 1 } else { 0 }),
        RuntimeValue::Int(n)     => mysql::Value::Int(*n),
        RuntimeValue::Float(f)   => mysql::Value::Float(*f as f32),
        RuntimeValue::String(s)  => mysql::Value::Bytes(s.as_bytes().to_vec()),
        other                    => mysql::Value::Bytes(format!("{}", other).into_bytes()),
    }
}

fn mysql_val_to_runtime(v: &mysql::Value) -> RuntimeValue {
    match v {
        mysql::Value::NULL           => RuntimeValue::Null,
        mysql::Value::Int(n)         => RuntimeValue::Int(*n),
        mysql::Value::UInt(n)        => RuntimeValue::Int(*n as i64),
        mysql::Value::Float(f)       => RuntimeValue::Float(*f as f64),
        mysql::Value::Double(f)      => RuntimeValue::Float(*f),
        mysql::Value::Bytes(b)       => RuntimeValue::String(
            String::from_utf8_lossy(b).into_owned()
        ),
        mysql::Value::Date(y,mo,d,h,mi,s,_) => RuntimeValue::String(
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)
        ),
        mysql::Value::Time(neg,d,h,m,s,_) => RuntimeValue::String(
            format!("{}{}:{:02}:{:02}", if *neg { "-" } else { "" }, d*24 + *h as u32, m, s)
        ),
    }
}
