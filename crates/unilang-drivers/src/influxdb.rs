// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! InfluxDB 2.x driver via HTTP (`ureq`).
//!
//! Communicates over the InfluxDB v2 REST API using the line-protocol write
//! endpoint and Flux query endpoint.  Compatible with InfluxDB Cloud as well
//! as self-hosted InfluxDB 2.x.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `influxdb_connect(url, token, org, bucket)` | Store connection config |
//! | `influxdb_write(measurement, tags, fields, timestamp_ns?)` | Write a single data point |
//! | `influxdb_query(flux_query)` | Execute a Flux query; returns raw CSV string |
//! | `influxdb_ping()` | Health-check (`GET /ping`); returns Bool |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct InfluxState {
    url: String,
    token: String,
    org: String,
    bucket: String,
}

pub struct InfluxDbDriver {
    state: Arc<Mutex<Option<InfluxState>>>,
}

impl InfluxDbDriver {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for InfluxDbDriver {
    fn default() -> Self {
        Self::new()
    }
}

// ── Trait impl ───────────────────────────────────────────────────────────────

impl UniLangDriver for InfluxDbDriver {
    fn name(&self) -> &str {
        "influxdb"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "InfluxDB 2.x time-series database via REST HTTP (ureq)"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Other
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "influxdb_connect",
            "influxdb_write",
            "influxdb_query",
            "influxdb_ping",
        ]
    }

    fn register(&self, vm: &mut VM) {
        macro_rules! arc {
            () => {
                Arc::clone(&self.state)
            };
        }

        // influxdb_connect(url, token, org, bucket)
        {
            let state = arc!();
            vm.register_builtin("influxdb_connect", move |args| {
                let url = str_arg(args, 0, "influxdb_connect(url, token, org, bucket)")?;
                let token = str_arg(args, 1, "influxdb_connect(url, token, org, bucket)")?;
                let org = str_arg(args, 2, "influxdb_connect(url, token, org, bucket)")?;
                let bucket = str_arg(args, 3, "influxdb_connect(url, token, org, bucket)")?;
                *state.lock().unwrap() = Some(InfluxState {
                    url: url.trim_end_matches('/').to_string(),
                    token,
                    org,
                    bucket,
                });
                Ok(RuntimeValue::Bool(true))
            });
        }

        // influxdb_write(measurement, tags, fields, timestamp_ns?)
        {
            let state = arc!();
            vm.register_builtin("influxdb_write", move |args| {
                let measurement =
                    str_arg(args, 0, "influxdb_write(measurement, tags, fields, ts?)")?;

                let tags = dict_to_pairs(args.get(1), "influxdb_write: tags")?;
                let fields = dict_to_pairs(args.get(2), "influxdb_write: fields")?;

                let timestamp_ns = match args.get(3) {
                    Some(RuntimeValue::Int(n)) => Some(*n),
                    Some(RuntimeValue::Float(f)) => Some(*f as i64),
                    Some(RuntimeValue::String(s)) => s.parse::<i64>().ok(),
                    _ => None,
                };

                // Build line-protocol: measurement,tag1=v1 field1=v1i,field2="str" ts
                let line = build_line_protocol(&measurement, &tags, &fields, timestamp_ns)?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("influxdb_write"))?;

                let write_url = format!(
                    "{}/api/v2/write?org={}&bucket={}&precision=ns",
                    cfg.url,
                    url_encode(&cfg.org),
                    url_encode(&cfg.bucket),
                );

                ureq::post(&write_url)
                    .set("Authorization", &format!("Token {}", cfg.token))
                    .set("Content-Type", "text/plain; charset=utf-8")
                    .send_string(&line)
                    .map_err(|e| RuntimeError::type_error(format!("influxdb_write: {}", e)))?;

                Ok(RuntimeValue::Bool(true))
            });
        }

        // influxdb_query(flux_query)
        {
            let state = arc!();
            vm.register_builtin("influxdb_query", move |args| {
                let flux_query = str_arg(args, 0, "influxdb_query(flux_query)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("influxdb_query"))?;

                let query_url = format!("{}/api/v2/query?org={}", cfg.url, url_encode(&cfg.org));
                let body = serde_json::json!({
                    "query": flux_query,
                    "type": "flux"
                })
                .to_string();

                let resp = ureq::post(&query_url)
                    .set("Authorization", &format!("Token {}", cfg.token))
                    .set("Content-Type", "application/json")
                    .set("Accept", "application/csv")
                    .send_string(&body)
                    .map_err(|e| RuntimeError::type_error(format!("influxdb_query: {}", e)))?;

                let csv = resp.into_string().map_err(|e| {
                    RuntimeError::type_error(format!("influxdb_query: read response: {}", e))
                })?;

                Ok(RuntimeValue::String(csv))
            });
        }

        // influxdb_ping()
        {
            let state = arc!();
            vm.register_builtin("influxdb_ping", move |args| {
                let _ = args; // no arguments
                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("influxdb_ping"))?;

                let ping_url = format!("{}/ping", cfg.url);
                let ok = ureq::get(&ping_url).call().is_ok();
                Ok(RuntimeValue::Bool(ok))
            });
        }
    }
}

// ── Line-protocol builder ─────────────────────────────────────────────────────

/// Build an InfluxDB line-protocol line.
///
/// Format: `measurement,tag1=v1,tag2=v2 field1=1i,field2="str",field3=3.14 timestamp_ns`
fn build_line_protocol(
    measurement: &str,
    tags: &[(String, String)],
    fields: &[(String, String)],
    timestamp_ns: Option<i64>,
) -> Result<String, RuntimeError> {
    if fields.is_empty() {
        return Err(RuntimeError::type_error(
            "influxdb_write: fields dict must not be empty",
        ));
    }

    let mut line = lp_escape_name(measurement);

    // Tag set (optional)
    if !tags.is_empty() {
        let tag_str: String = tags
            .iter()
            .map(|(k, v)| format!("{}={}", lp_escape_name(k), lp_escape_tag_value(v)))
            .collect::<Vec<_>>()
            .join(",");
        line.push(',');
        line.push_str(&tag_str);
    }

    // Field set (required)
    let field_str: String = fields
        .iter()
        .map(|(k, v)| format!("{}={}", lp_escape_name(k), lp_field_value(v)))
        .collect::<Vec<_>>()
        .join(",");
    line.push(' ');
    line.push_str(&field_str);

    // Optional timestamp
    if let Some(ts) = timestamp_ns {
        line.push(' ');
        line.push_str(&ts.to_string());
    }

    Ok(line)
}

/// Escape measurement/key names (escape commas, spaces, equals signs).
fn lp_escape_name(s: &str) -> String {
    s.replace(',', "\\,")
        .replace(' ', "\\ ")
        .replace('=', "\\=")
}

/// Escape tag values (same rules as names).
fn lp_escape_tag_value(s: &str) -> String {
    lp_escape_name(s)
}

/// Format a field value.  Integers get the `i` suffix; strings get quotes;
/// floats are written as-is; booleans become `t`/`f`.
fn lp_field_value(v: &str) -> String {
    // Try integer
    if let Ok(n) = v.parse::<i64>() {
        return format!("{}i", n);
    }
    // Try float
    if let Ok(f) = v.parse::<f64>() {
        return format!("{}", f);
    }
    // Boolean
    let lower = v.to_lowercase();
    if lower == "true" || lower == "false" {
        return lower;
    }
    // Default: string field — escape backslash and double-quote
    let escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// Minimal percent-encoding for query-string values (encodes space, #, %, &, +, =).
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

// ── Dict helper ───────────────────────────────────────────────────────────────

/// Convert a Dict RuntimeValue into `Vec<(key_string, value_string)>`.
/// An absent or Null value yields an empty vec.
fn dict_to_pairs(
    v: Option<&RuntimeValue>,
    ctx: &str,
) -> Result<Vec<(String, String)>, RuntimeError> {
    match v {
        None | Some(RuntimeValue::Null) => Ok(vec![]),
        Some(RuntimeValue::Dict(pairs)) => {
            let mut out = Vec::with_capacity(pairs.len());
            for (k, val) in pairs {
                let key = match k {
                    RuntimeValue::String(s) => s.clone(),
                    other => format!("{}", other),
                };
                let value = match val {
                    RuntimeValue::String(s) => s.clone(),
                    RuntimeValue::Int(n) => n.to_string(),
                    RuntimeValue::Float(f) => f.to_string(),
                    RuntimeValue::Bool(b) => b.to_string(),
                    RuntimeValue::Null => "null".to_string(),
                    other => format!("{}", other),
                };
                out.push((key, value));
            }
            Ok(out)
        }
        Some(other) => Err(RuntimeError::type_error(format!(
            "{}: expected Dict, got {:?}",
            ctx, other
        ))),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call influxdb_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!(
            "{}: expected string at position {}",
            sig, idx
        ))),
    }
}
