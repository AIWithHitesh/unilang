// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # UniLang Driver Ecosystem
//!
//! This crate provides the **driver framework** for UniLang and ships all
//! first-party drivers for databases, caches, queues, and search engines.
//!
//! ## Architecture
//!
//! Every driver implements the [`UniLangDriver`] trait and registers its
//! built-in functions with the VM via `Arc<Mutex<>>` state so that closures
//! can hold connection state without touching the VM internals.
//!
//! ```text
//! UniLang script
//!     │  redis_set("key", "value")
//!     ▼
//! VM::call_builtin("redis_set", args)
//!     │  registered by RedisDriver::register()
//!     ▼
//! Arc<Mutex<RedisConnection>>::lock() → actual Redis call
//! ```
//!
//! ## Adding a new driver
//!
//! 1. Create `src/my_driver.rs` implementing [`UniLangDriver`].
//! 2. Add an optional feature in `Cargo.toml`.
//! 3. Re-export under a `#[cfg(feature = "…")]` guard in this file.
//! 4. Register in `unilang-cli/src/main.rs`.
//! 5. Add exported function names to the semantic analyser prelude.
//! 6. Open a PR — see `CONTRIBUTING.md`.

use unilang_runtime::vm::VM;

// ── Driver trait ─────────────────────────────────────────────────────────────

/// Category of a driver — used for documentation and tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverCategory {
    /// Relational / SQL databases
    SqlDatabase,
    /// Document / key-value / wide-column NoSQL stores
    NoSqlDatabase,
    /// In-process or remote caching layers
    Cache,
    /// Message queues and streaming platforms
    Queue,
    /// Full-text search and indexing engines
    Search,
    /// Workflow orchestrators, state machines, schedulers
    Workflow,
    /// Anything else
    Other,
}

/// The core trait every UniLang driver must implement.
///
/// Drivers are stateful objects that hold connection handles behind
/// `Arc<Mutex<>>`. The VM does not know about connection details; it only
/// sees the [`RuntimeValue`]-typed functions registered via
/// [`VM::register_builtin`].
///
/// [`RuntimeValue`]: unilang_runtime::value::RuntimeValue
pub trait UniLangDriver: Send + Sync {
    /// Short machine-readable name (e.g. `"sqlite"`, `"redis"`).
    fn name(&self) -> &str;

    /// SemVer version string of this driver (e.g. `"1.0.0"`).
    fn version(&self) -> &str;

    /// Human-readable description shown in `unilang drivers list`.
    fn description(&self) -> &str;

    /// Category for grouping in documentation and tooling.
    fn category(&self) -> DriverCategory;

    /// Names of every built-in function this driver exports.
    ///
    /// Used by the semantic analyser to pre-populate the global scope so
    /// that scripts type-check correctly before running.
    fn exported_functions(&self) -> &'static [&'static str];

    /// Register all built-in functions into `vm`.
    ///
    /// Called once at VM startup. Implementations should clone any
    /// `Arc<Mutex<>>` state they need and capture it in the closure.
    fn register(&self, vm: &mut VM);
}

// ── Driver registry ───────────────────────────────────────────────────────────

/// Central registry that collects every driver and can batch-register them.
///
/// # Example
/// ```rust,ignore
/// let mut registry = DriverRegistry::new();
/// registry.add(Box::new(SqliteDriver::new()));
/// registry.add(Box::new(RedisDriver::new()));
/// registry.register_all(&mut vm);
/// let func_names = registry.all_function_names(); // feed to semantic analyser
/// ```
pub struct DriverRegistry {
    drivers: Vec<Box<dyn UniLangDriver>>,
}

impl DriverRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { drivers: Vec::new() }
    }

    /// Add a driver to the registry.
    pub fn add(&mut self, driver: Box<dyn UniLangDriver>) {
        self.drivers.push(driver);
    }

    /// Register every driver's built-in functions with the VM.
    pub fn register_all(&self, vm: &mut VM) {
        for driver in &self.drivers {
            driver.register(vm);
        }
    }

    /// Collect all exported function names from every registered driver.
    ///
    /// Pass the returned `Vec<String>` to the semantic analyser so it
    /// recognises driver functions during type-checking.
    pub fn all_function_names(&self) -> Vec<String> {
        self.drivers
            .iter()
            .flat_map(|d| d.exported_functions().iter().map(|s| s.to_string()))
            .collect()
    }

    /// List every driver by name, version, and category.
    pub fn list(&self) -> Vec<(&str, &str, DriverCategory)> {
        self.drivers
            .iter()
            .map(|d| (d.name(), d.version(), d.category()))
            .collect()
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Built-in driver modules ───────────────────────────────────────────────────

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql-driver")]
pub mod mysql;

#[cfg(feature = "postgres-driver")]
pub mod postgres;

#[cfg(feature = "mongodb-driver")]
pub mod mongodb;

#[cfg(feature = "redis-driver")]
pub mod redis;

#[cfg(feature = "kafka")]
pub mod kafka;

#[cfg(feature = "elasticsearch-driver")]
pub mod elasticsearch;

#[cfg(feature = "memcached-driver")]
pub mod memcached;

// ── Convenience: build the default registry ──────────────────────────────────

/// Build a [`DriverRegistry`] pre-populated with every driver that was
/// compiled in (determined by Cargo feature flags).
///
/// This is the function to call from `unilang-cli`.
pub fn default_registry() -> DriverRegistry {
    let mut r = DriverRegistry::new();

    #[cfg(feature = "sqlite")]
    r.add(Box::new(sqlite::SqliteDriver::new()));

    #[cfg(feature = "mysql-driver")]
    r.add(Box::new(mysql::MySqlDriver::new()));

    #[cfg(feature = "postgres-driver")]
    r.add(Box::new(postgres::PostgresDriver::new()));

    #[cfg(feature = "mongodb-driver")]
    r.add(Box::new(mongodb::MongoDriver::new()));

    #[cfg(feature = "redis-driver")]
    r.add(Box::new(redis::RedisDriver::new()));

    #[cfg(feature = "kafka")]
    r.add(Box::new(kafka::KafkaDriver::new()));

    #[cfg(feature = "elasticsearch-driver")]
    r.add(Box::new(elasticsearch::ElasticsearchDriver::new()));

    #[cfg(feature = "memcached-driver")]
    r.add(Box::new(memcached::MemcachedDriver::new()));

    r
}
