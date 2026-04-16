// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Elasticsearch / OpenSearch driver via HTTP (`ureq`).
//!
//! No native library required — communicates over REST, so it works with any
//! Elasticsearch-compatible endpoint (Elasticsearch 7/8, OpenSearch, etc.).
//!
//! # UniLang functions
//! | Function | Signature | Description |
//! |---|---|---|
//! | `es_connect` | `es_connect(url)` | Set base URL — e.g. `"http://localhost:9200"` |
//! | `es_index` | `es_index(index, id, doc_json)` | Index (upsert) a document |
//! | `es_get` | `es_get(index, id)` | Retrieve a document by id → `Dict` or `null` |
//! | `es_search` | `es_search(index, query_json)` | Search → `List[Dict]` (hits) |
//! | `es_delete` | `es_delete(index, id)` | Delete a document |
//! | `es_create_index` | `es_create_index(index, settings_json?)` | Create an index |
//! | `es_delete_index` | `es_delete_index(index)` | Delete an index |
//! | `es_count` | `es_count(index, query_json?)` | Count matching documents |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

pub struct ElasticsearchDriver {
    base_url: Arc<Mutex<String>>,
}

impl ElasticsearchDriver {
    pub fn new() -> Self {
        Self { base_url: Arc::new(Mutex::new("http://localhost:9200".to_string())) }
    }
}

impl Default for ElasticsearchDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for ElasticsearchDriver {
    fn name(&self) -> &str { "elasticsearch" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Elasticsearch / OpenSearch via REST HTTP (ureq)" }
    fn category(&self) -> DriverCategory { DriverCategory::Search }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "es_connect",
            "es_index", "es_get", "es_search", "es_delete",
            "es_create_index", "es_delete_index", "es_count",
        ]
    }

    fn register(&self, vm: &mut VM) {
        // es_connect(url)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_connect", move |args| {
                let url = str_arg(args, 0, "es_connect(url)")?;
                *base.lock().unwrap() = url.trim_end_matches('/').to_string();
                Ok(RuntimeValue::Bool(true))
            });
        }

        // es_index(index, id, doc_json)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_index", move |args| {
                let index    = str_arg(args, 0, "es_index(index, id, doc)")?;
                let id       = str_arg(args, 1, "es_index(index, id, doc)")?;
                let doc_json = str_arg(args, 2, "es_index(index, id, doc)")?;
                let url = format!("{}/{}/_doc/{}", base.lock().unwrap(), index, id);
                let resp = ureq::put(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&doc_json)
                    .map_err(|e| RuntimeError::type_error(format!("es_index: {}", e)))?;
                let body = resp.into_string().unwrap_or_default();
                Ok(RuntimeValue::String(body))
            });
        }

        // es_get(index, id)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_get", move |args| {
                let index = str_arg(args, 0, "es_get(index, id)")?;
                let id    = str_arg(args, 1, "es_get(index, id)")?;
                let url   = format!("{}/{}/_doc/{}", base.lock().unwrap(), index, id);
                let result = ureq::get(&url).call();
                match result {
                    Ok(resp) => {
                        let body = resp.into_string().unwrap_or_default();
                        match serde_json::from_str::<serde_json::Value>(&body) {
                            Ok(v) => {
                                // ES wraps the doc in _source
                                let source = v.get("_source").unwrap_or(&v);
                                Ok(json_value_to_runtime(source))
                            }
                            Err(_) => Ok(RuntimeValue::Null),
                        }
                    }
                    Err(ureq::Error::Status(404, _)) => Ok(RuntimeValue::Null),
                    Err(e) => Err(RuntimeError::type_error(format!("es_get: {}", e))),
                }
            });
        }

        // es_search(index, query_json)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_search", move |args| {
                let index      = str_arg(args, 0, "es_search(index, query)")?;
                let query_json = match args.get(1) {
                    Some(RuntimeValue::String(s)) => s.clone(),
                    _ => r#"{"query":{"match_all":{}}}"#.to_string(),
                };
                let url = format!("{}/{}/_search", base.lock().unwrap(), index);
                let body = ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&query_json)
                    .map_err(|e| RuntimeError::type_error(format!("es_search: {}", e)))?
                    .into_string()
                    .unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&body)
                    .unwrap_or(serde_json::Value::Null);
                let hits = parsed
                    .pointer("/hits/hits")
                    .and_then(|h| h.as_array())
                    .cloned()
                    .unwrap_or_default();
                let results: Vec<RuntimeValue> = hits.iter()
                    .map(|hit| {
                        hit.get("_source").map(json_value_to_runtime)
                            .unwrap_or(RuntimeValue::Null)
                    })
                    .collect();
                Ok(RuntimeValue::List(results))
            });
        }

        // es_delete(index, id)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_delete", move |args| {
                let index = str_arg(args, 0, "es_delete(index, id)")?;
                let id    = str_arg(args, 1, "es_delete(index, id)")?;
                let url   = format!("{}/{}/_doc/{}", base.lock().unwrap(), index, id);
                ureq::delete(&url).call()
                    .map_err(|e| RuntimeError::type_error(format!("es_delete: {}", e)))?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // es_create_index(index, settings_json?)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_create_index", move |args| {
                let index    = str_arg(args, 0, "es_create_index(index, settings?)")?;
                let settings = match args.get(1) {
                    Some(RuntimeValue::String(s)) => s.clone(),
                    _ => "{}".to_string(),
                };
                let url = format!("{}/{}", base.lock().unwrap(), index);
                ureq::put(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&settings)
                    .map_err(|e| RuntimeError::type_error(format!("es_create_index: {}", e)))?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // es_delete_index(index)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_delete_index", move |args| {
                let index = str_arg(args, 0, "es_delete_index(index)")?;
                let url   = format!("{}/{}", base.lock().unwrap(), index);
                ureq::delete(&url).call()
                    .map_err(|e| RuntimeError::type_error(format!("es_delete_index: {}", e)))?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // es_count(index, query_json?)
        {
            let base = Arc::clone(&self.base_url);
            vm.register_builtin("es_count", move |args| {
                let index      = str_arg(args, 0, "es_count(index, query?)")?;
                let query_json = match args.get(1) {
                    Some(RuntimeValue::String(s)) => s.clone(),
                    _ => r#"{"query":{"match_all":{}}}"#.to_string(),
                };
                let url = format!("{}/{}/_count", base.lock().unwrap(), index);
                let body = ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&query_json)
                    .map_err(|e| RuntimeError::type_error(format!("es_count: {}", e)))?
                    .into_string()
                    .unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&body)
                    .unwrap_or(serde_json::Value::Null);
                let count = parsed.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
                Ok(RuntimeValue::Int(count))
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!("{}: expected string at position {}", sig, idx))),
    }
}

fn json_value_to_runtime(v: &serde_json::Value) -> RuntimeValue {
    match v {
        serde_json::Value::Null        => RuntimeValue::Null,
        serde_json::Value::Bool(b)     => RuntimeValue::Bool(*b),
        serde_json::Value::Number(n)   => {
            if let Some(i) = n.as_i64() { RuntimeValue::Int(i) }
            else { RuntimeValue::Float(n.as_f64().unwrap_or(0.0)) }
        }
        serde_json::Value::String(s)   => RuntimeValue::String(s.clone()),
        serde_json::Value::Array(arr)  => {
            RuntimeValue::List(arr.iter().map(json_value_to_runtime).collect())
        }
        serde_json::Value::Object(obj) => {
            let pairs = obj.iter()
                .map(|(k, v)| (RuntimeValue::String(k.clone()), json_value_to_runtime(v)))
                .collect();
            RuntimeValue::Dict(pairs)
        }
    }
}
