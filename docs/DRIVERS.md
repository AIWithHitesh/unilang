# UniLang Driver Development Guide

The `unilang-drivers` crate is UniLang's official connectivity layer. It provides a plugin-style architecture for adding drivers for databases, caches, queues, search engines, and anything else that needs external I/O — without modifying the VM or standard library.

---

## Architecture

```
unilang-cli
    │
    └── unilang_drivers::default_registry()
              │
              └── DriverRegistry
                      │
                      ├── SqliteDriver  ──→ vm.register_builtin("db_connect", ...)
                      ├── RedisDriver   ──→ vm.register_builtin("redis_get", ...)
                      ├── KafkaDriver   ──→ vm.register_builtin("kafka_produce", ...)
                      └── ...
```

The VM has no knowledge of any specific driver. Each driver implements the `UniLangDriver` trait and registers one or more named builtins with the VM. When UniLang code calls `db_query(...)`, the VM dispatches to the closure that `SqliteDriver` registered — exactly like any other builtin.

---

## The `UniLangDriver` Trait

```rust
pub trait UniLangDriver: Send + Sync {
    /// Short identifier (e.g. "sqlite", "redis").
    fn name(&self) -> &'static str;

    /// SemVer string.
    fn version(&self) -> &'static str;

    /// One-line description shown in `unilang drivers list`.
    fn description(&self) -> &'static str;

    /// Category for grouping in listings.
    fn category(&self) -> DriverCategory;

    /// All function names this driver exports into UniLang scope.
    /// These must also be listed in the semantic analyzer's PRELUDE_FUNCS.
    fn exported_functions(&self) -> &'static [&'static str];

    /// Register all builtins with the VM.
    fn register(&self, vm: &mut unilang_runtime::vm::VM);
}
```

---

## Bundled Drivers

### Default Features (enabled automatically)

| Driver | Feature flag | Crate | Functions |
|--------|-------------|-------|-----------|
| SQLite | `sqlite` | `rusqlite 0.32` (bundled) | `db_connect`, `db_query`, `db_exec` |
| Redis | `redis-driver` | `redis 0.27` | `redis_connect`, `redis_get`, `redis_set`, `redis_setex`, `redis_del`, `redis_exists`, `redis_expire`, `redis_ttl`, `redis_incr`, `redis_decr`, `redis_lpush`, `redis_lrange`, `redis_hset`, `redis_hget`, `redis_hdel`, `redis_hgetall`, `redis_sadd`, `redis_smembers` |
| Kafka | `kafka` | in-memory (no broker) | `kafka_connect`, `kafka_produce`, `kafka_events`, `kafka_clear` |
| Elasticsearch | `elasticsearch-driver` | `ureq 2` (HTTP) | `es_connect`, `es_index`, `es_get`, `es_search`, `es_delete`, `es_create_index`, `es_delete_index`, `es_count` |

### Optional Features (require Rust ≥ 1.85)

| Driver | Feature flag | Crate | Functions |
|--------|-------------|-------|-----------|
| MySQL | `mysql-driver` | `mysql 25` | `mysql_connect`, `mysql_query`, `mysql_exec`, `mysql_close` |
| PostgreSQL | `postgres-driver` | `postgres 0.19` | `pg_connect`, `pg_query`, `pg_exec`, `pg_close` |
| MongoDB | `mongodb-driver` | `mongodb 2` (sync) | `mongo_connect`, `mongo_db`, `mongo_find`, `mongo_find_one`, `mongo_insert`, `mongo_update`, `mongo_delete`, `mongo_count` |
| Memcached | `memcached-driver` | `memcache 0.17` | `memcached_connect`, `memcached_get`, `memcached_set`, `memcached_set_with_ttl`, `memcached_delete`, `memcached_flush`, `memcached_incr`, `memcached_decr`, `memcached_stats` |

Enable optional drivers:
```toml
# In the CLI or your own crate:
unilang-drivers = { path = "../unilang-drivers", features = ["mysql-driver", "postgres-driver"] }
```

Or enable everything:
```toml
unilang-drivers = { path = "../unilang-drivers", features = ["all-drivers"] }
```

---

## Writing Your Own Driver

### Step 1 — Understand the value model

UniLang values passed into builtins are `unilang_runtime::value::Value`:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Dict(IndexMap<String, Value>),
    // ...
}
```

Builtin functions have the signature:
```rust
type Builtin = fn(Vec<Value>) -> Result<Value, RuntimeError>;
```

### Step 2 — Create the driver file

```rust
// crates/unilang-drivers/src/mydb.rs
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

use std::sync::{Arc, Mutex};
use unilang_runtime::vm::VM;
use unilang_runtime::value::Value;
use unilang_runtime::error::{RuntimeError, ErrorKind};
use crate::{DriverCategory, UniLangDriver};

pub struct MyDbDriver {
    // Wrap connection state in Arc<Mutex<Option<...>>> so it can be
    // cloned into multiple builtin closures.
    conn: Arc<Mutex<Option<mydb::Connection>>>,
}

impl MyDbDriver {
    pub fn new() -> Self {
        Self { conn: Arc::new(Mutex::new(None)) }
    }
}

impl UniLangDriver for MyDbDriver {
    fn name(&self)        -> &'static str { "mydb" }
    fn version(&self)     -> &'static str { "1.0.0" }
    fn description(&self) -> &'static str { "MyDB driver for UniLang" }
    fn category(&self)    -> DriverCategory { DriverCategory::Database }

    fn exported_functions(&self) -> &'static [&'static str] {
        &["mydb_connect", "mydb_query", "mydb_close"]
    }

    fn register(&self, vm: &mut VM) {
        // ── mydb_connect(url) ──────────────────────────────────────
        let conn = Arc::clone(&self.conn);
        vm.register_builtin("mydb_connect", move |args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err(RuntimeError {
                    kind: ErrorKind::RuntimeError,
                    message: "mydb_connect(url): url must be a string".into(),
                }),
            };
            let c = mydb::Connection::open(&url).map_err(|e| RuntimeError {
                kind: ErrorKind::RuntimeError,
                message: format!("mydb_connect: {}", e),
            })?;
            *conn.lock().unwrap() = Some(c);
            Ok(Value::Bool(true))
        });

        // ── mydb_query(sql, params) ────────────────────────────────
        let conn = Arc::clone(&self.conn);
        vm.register_builtin("mydb_query", move |args| {
            let guard = conn.lock().unwrap();
            let c = guard.as_ref().ok_or_else(|| RuntimeError {
                kind: ErrorKind::RuntimeError,
                message: "mydb_query: not connected — call mydb_connect first".into(),
            })?;
            let sql = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err(RuntimeError {
                    kind: ErrorKind::RuntimeError,
                    message: "mydb_query(sql): sql must be a string".into(),
                }),
            };
            // ... execute query, convert rows to Vec<Value::Dict>
            Ok(Value::Array(vec![]))
        });

        // ── mydb_close() ──────────────────────────────────────────
        let conn = Arc::clone(&self.conn);
        vm.register_builtin("mydb_close", move |_args| {
            *conn.lock().unwrap() = None;
            Ok(Value::Bool(true))
        });
    }
}
```

### Step 3 — Add the Cargo dependency (feature-gated)

```toml
# crates/unilang-drivers/Cargo.toml

[dependencies]
mydb = { version = "1", optional = true }

[features]
mydb-driver = ["dep:mydb"]
all-drivers = [
    "sqlite",
    "redis-driver",
    "kafka",
    "elasticsearch-driver",
    "mydb-driver",   # add here
]
```

### Step 4 — Register in lib.rs

```rust
// crates/unilang-drivers/src/lib.rs

#[cfg(feature = "mydb-driver")]
mod mydb;

pub fn default_registry() -> DriverRegistry {
    let mut r = DriverRegistry::new();

    // existing drivers ...

    #[cfg(feature = "mydb-driver")]
    r.register(Box::new(mydb::MyDbDriver::new()));

    r
}
```

### Step 5 — Add function names to the semantic analyzer

Open `crates/unilang-semantic/src/analyzer.rs` and add your exported function names to `PRELUDE_FUNCS`:

```rust
static PRELUDE_FUNCS: &[&str] = &[
    // ... existing entries ...
    "mydb_connect",
    "mydb_query",
    "mydb_close",
];
```

This tells the type-checker that these names are valid — otherwise scripts will fail with `undefined variable 'mydb_connect'` at the semantic analysis phase.

### Step 6 — Write a usage example

Create `examples/<yourdb>/server.uniL`:

```python
mydb_connect("mydb://localhost/testdb")

rows = mydb_query("SELECT id, name FROM users WHERE active = 1")
for row in rows:
    print(row["name"])

mydb_close()
```

---

## Design Guidelines

**Keep state in `Arc<Mutex<Option<Connection>>>`**
Don't store state in the VM. Each driver owns its connection pool behind an Arc so the same data is accessible from every registered closure.

**Return `Value::Null` for void operations**
`db_exec`, `redis_del`, `kafka_produce` etc. return `Value::Bool(true)` on success or a descriptive `RuntimeError` on failure. Never return `Value::Null` from an operation that can fail silently.

**Fail loudly with context**
Error messages should say what went wrong and what the caller should do:
```
"redis_get: not connected — call redis_connect(host, port) first"
```

**Type-check arguments defensively**
Validate arg count and types at the top of each builtin, before touching the connection.

**Feature-gate heavy dependencies**
If your crate adds significant compile time or has platform-specific build requirements, put it behind an optional feature flag. Light HTTP-based drivers (like Elasticsearch via `ureq`) can be in default features.

---

## Testing Your Driver

The simplest way to test a driver is to write a `.uniL` script:

```bash
# From repo root
cargo build --release --features mydb-driver
./target/release/unilang run examples/mydb/test.uniL
```

For automated testing, add a Rust integration test in `crates/unilang-drivers/tests/`:

```rust
#[test]
#[cfg(feature = "mydb-driver")]
fn test_mydb_round_trip() {
    use unilang_runtime::vm::VM;
    use unilang_runtime::value::Value;
    use unilang_drivers::default_registry;

    let mut vm = VM::new();
    let registry = default_registry();
    registry.register_all(&mut vm);

    // Call mydb_connect
    let result = vm.call_builtin("mydb_connect", vec![
        Value::String(":memory:".into()),
    ]);
    assert!(result.is_ok());
}
```

---

## Questions?

Open a GitHub Discussion tagged `drivers` or check existing driver implementations as reference:
- Simple stateful: [`src/sqlite.rs`](../crates/unilang-drivers/src/sqlite.rs)
- In-memory queue: [`src/kafka.rs`](../crates/unilang-drivers/src/kafka.rs)
- HTTP-based: [`src/elasticsearch.rs`](../crates/unilang-drivers/src/elasticsearch.rs)
