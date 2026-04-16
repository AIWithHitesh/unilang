// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! MongoDB driver via `mongodb` 2.x (synchronous `sync` module).
//!
//! # UniLang functions
//! | Function | Signature | Description |
//! |---|---|---|
//! | `mongo_connect` | `mongo_connect(url)` | Connect — e.g. `"mongodb://localhost:27017"` |
//! | `mongo_db` | `mongo_db(name)` | Select database |
//! | `mongo_find` | `mongo_find(collection, filter_json?)` | Find many → `List[Dict]` |
//! | `mongo_find_one` | `mongo_find_one(collection, filter_json)` | Find one → `Dict` or `null` |
//! | `mongo_insert` | `mongo_insert(collection, doc_json)` | Insert one → inserted id |
//! | `mongo_update` | `mongo_update(collection, filter_json, update_json)` | Update many → count |
//! | `mongo_delete` | `mongo_delete(collection, filter_json)` | Delete many → count |
//! | `mongo_count` | `mongo_count(collection, filter_json?)` | Count documents |

use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

type MongoState = Option<(mongodb::sync::Client, String)>;

pub struct MongoDriver {
    state: Arc<Mutex<MongoState>>,
}

impl MongoDriver {
    pub fn new() -> Self {
        Self { state: Arc::new(Mutex::new(None)) }
    }
}

impl Default for MongoDriver {
    fn default() -> Self { Self::new() }
}

impl UniLangDriver for MongoDriver {
    fn name(&self) -> &str { "mongodb" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "MongoDB via mongodb 2.x sync driver" }
    fn category(&self) -> DriverCategory { DriverCategory::NoSqlDatabase }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "mongo_connect", "mongo_db",
            "mongo_find", "mongo_find_one",
            "mongo_insert", "mongo_update", "mongo_delete", "mongo_count",
        ]
    }

    fn register(&self, vm: &mut VM) {
        // mongo_connect(url)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_connect", move |args| {
                let url = str_arg(args, 0, "mongo_connect(url)")?;
                let client = mongodb::sync::Client::with_uri_str(&url)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_connect: {}", e)))?;
                let mut guard = state.lock().unwrap();
                let db = guard.as_ref().map(|(_, d)| d.clone()).unwrap_or_else(|| "test".to_string());
                *guard = Some((client, db));
                Ok(RuntimeValue::Bool(true))
            });
        }

        // mongo_db(name)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_db", move |args| {
                let name = str_arg(args, 0, "mongo_db(name)")?;
                let mut guard = state.lock().unwrap();
                if let Some((client, db)) = guard.as_mut() {
                    *db = name;
                    let _ = client; // keep borrow alive
                }
                Ok(RuntimeValue::Null)
            });
        }

        // mongo_find(collection, filter_json?)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_find", move |args| {
                let coll_name = str_arg(args, 0, "mongo_find(collection, filter?)")?;
                let filter = parse_filter(args, 1)?;
                let guard = state.lock().unwrap();
                let (client, db_name) = guard.as_ref().ok_or_else(|| no_conn("mongo_find"))?;
                let coll: mongodb::sync::Collection<mongodb::bson::Document> =
                    client.database(db_name).collection(&coll_name);
                let cursor = coll.find(filter, None)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_find: {}", e)))?;
                let docs: Vec<RuntimeValue> = cursor
                    .filter_map(|r| r.ok())
                    .map(|doc| bson_doc_to_runtime(&doc))
                    .collect();
                Ok(RuntimeValue::List(docs))
            });
        }

        // mongo_find_one(collection, filter_json)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_find_one", move |args| {
                let coll_name = str_arg(args, 0, "mongo_find_one(collection, filter)")?;
                let filter = parse_filter(args, 1)?;
                let guard = state.lock().unwrap();
                let (client, db_name) = guard.as_ref().ok_or_else(|| no_conn("mongo_find_one"))?;
                let coll: mongodb::sync::Collection<mongodb::bson::Document> =
                    client.database(db_name).collection(&coll_name);
                let doc = coll.find_one(filter, None)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_find_one: {}", e)))?;
                Ok(match doc {
                    Some(d) => bson_doc_to_runtime(&d),
                    None => RuntimeValue::Null,
                })
            });
        }

        // mongo_insert(collection, doc_json)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_insert", move |args| {
                let coll_name = str_arg(args, 0, "mongo_insert(collection, doc)")?;
                let doc_json  = str_arg(args, 1, "mongo_insert(collection, doc)")?;
                let doc: mongodb::bson::Document = serde_json::from_str(&doc_json)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_insert: bad JSON: {}", e)))?;
                let guard = state.lock().unwrap();
                let (client, db_name) = guard.as_ref().ok_or_else(|| no_conn("mongo_insert"))?;
                let coll: mongodb::sync::Collection<mongodb::bson::Document> =
                    client.database(db_name).collection(&coll_name);
                let result = coll.insert_one(doc, None)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_insert: {}", e)))?;
                Ok(RuntimeValue::String(result.inserted_id.to_string()))
            });
        }

        // mongo_update(collection, filter_json, update_json)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_update", move |args| {
                let coll_name   = str_arg(args, 0, "mongo_update(collection, filter, update)")?;
                let filter_json = str_arg(args, 1, "mongo_update(collection, filter, update)")?;
                let update_json = str_arg(args, 2, "mongo_update(collection, filter, update)")?;
                let filter: mongodb::bson::Document = serde_json::from_str(&filter_json)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_update filter: {}", e)))?;
                let update: mongodb::bson::Document = serde_json::from_str(&update_json)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_update update: {}", e)))?;
                let guard = state.lock().unwrap();
                let (client, db_name) = guard.as_ref().ok_or_else(|| no_conn("mongo_update"))?;
                let coll: mongodb::sync::Collection<mongodb::bson::Document> =
                    client.database(db_name).collection(&coll_name);
                let result = coll.update_many(filter, update, None)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_update: {}", e)))?;
                Ok(RuntimeValue::Int(result.modified_count as i64))
            });
        }

        // mongo_delete(collection, filter_json)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_delete", move |args| {
                let coll_name   = str_arg(args, 0, "mongo_delete(collection, filter)")?;
                let filter_json = str_arg(args, 1, "mongo_delete(collection, filter)")?;
                let filter: mongodb::bson::Document = serde_json::from_str(&filter_json)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_delete: {}", e)))?;
                let guard = state.lock().unwrap();
                let (client, db_name) = guard.as_ref().ok_or_else(|| no_conn("mongo_delete"))?;
                let coll: mongodb::sync::Collection<mongodb::bson::Document> =
                    client.database(db_name).collection(&coll_name);
                let result = coll.delete_many(filter, None)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_delete: {}", e)))?;
                Ok(RuntimeValue::Int(result.deleted_count as i64))
            });
        }

        // mongo_count(collection, filter_json?)
        {
            let state = Arc::clone(&self.state);
            vm.register_builtin("mongo_count", move |args| {
                let coll_name = str_arg(args, 0, "mongo_count(collection, filter?)")?;
                let filter    = parse_filter(args, 1)?;
                let guard = state.lock().unwrap();
                let (client, db_name) = guard.as_ref().ok_or_else(|| no_conn("mongo_count"))?;
                let coll: mongodb::sync::Collection<mongodb::bson::Document> =
                    client.database(db_name).collection(&coll_name);
                let count = coll.count_documents(filter, None)
                    .map_err(|e| RuntimeError::type_error(format!("mongo_count: {}", e)))?;
                Ok(RuntimeValue::Int(count as i64))
            });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call mongo_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!("{}: expected string at position {}", sig, idx))),
    }
}

fn parse_filter(
    args: &[RuntimeValue],
    idx: usize,
) -> Result<Option<mongodb::bson::Document>, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) if !s.is_empty() => {
            let doc: mongodb::bson::Document = serde_json::from_str(s)
                .map_err(|e| RuntimeError::type_error(format!("filter JSON: {}", e)))?;
            Ok(Some(doc))
        }
        _ => Ok(None),
    }
}

fn bson_val_to_runtime(v: &mongodb::bson::Bson) -> RuntimeValue {
    use mongodb::bson::Bson;
    match v {
        Bson::Null              => RuntimeValue::Null,
        Bson::Boolean(b)        => RuntimeValue::Bool(*b),
        Bson::Int32(n)          => RuntimeValue::Int(*n as i64),
        Bson::Int64(n)          => RuntimeValue::Int(*n),
        Bson::Double(f)         => RuntimeValue::Float(*f),
        Bson::String(s)         => RuntimeValue::String(s.clone()),
        Bson::Array(arr)        => RuntimeValue::List(arr.iter().map(bson_val_to_runtime).collect()),
        Bson::Document(doc)     => bson_doc_to_runtime(doc),
        Bson::ObjectId(oid)     => RuntimeValue::String(oid.to_hex()),
        Bson::DateTime(dt)      => RuntimeValue::String(dt.to_string()),
        other                   => RuntimeValue::String(other.to_string()),
    }
}

fn bson_doc_to_runtime(doc: &mongodb::bson::Document) -> RuntimeValue {
    let pairs: Vec<(RuntimeValue, RuntimeValue)> = doc.iter()
        .map(|(k, v)| (RuntimeValue::String(k.clone()), bson_val_to_runtime(v)))
        .collect();
    RuntimeValue::Dict(pairs)
}
