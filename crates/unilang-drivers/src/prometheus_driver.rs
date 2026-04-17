// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Prometheus metrics driver — in-process metrics collection and exposition.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `prom_counter(name, help)` | Register a counter metric |
//! | `prom_gauge(name, help)` | Register a gauge metric |
//! | `prom_histogram(name, help)` | Register a histogram metric with default buckets |
//! | `prom_counter_inc(name, by?)` | Increment counter by 1 or given amount |
//! | `prom_gauge_set(name, value)` | Set gauge to a value |
//! | `prom_gauge_inc(name, by?)` | Increment gauge |
//! | `prom_gauge_dec(name, by?)` | Decrement gauge |
//! | `prom_histogram_observe(name, value)` | Record an observation in a histogram |
//! | `prom_export()` | Return metrics in Prometheus text format |
//! | `prom_serve(port)` | Start a background HTTP server on /metrics |

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

#[cfg(feature = "prometheus-driver")]
use prometheus::{Counter, Encoder, Gauge, Histogram, Registry, TextEncoder};

/// Each named metric is one of these variants.
#[cfg(feature = "prometheus-driver")]
enum MetricValue {
    Counter(Counter),
    Gauge(Gauge),
    Histogram(Histogram),
}

#[cfg(feature = "prometheus-driver")]
struct PrometheusState {
    registry: Registry,
    metrics: HashMap<String, MetricValue>,
}

#[cfg(feature = "prometheus-driver")]
impl PrometheusState {
    fn new() -> Self {
        Self {
            registry: Registry::new(),
            metrics: HashMap::new(),
        }
    }
}

pub struct PrometheusDriver {
    #[cfg(feature = "prometheus-driver")]
    state: Arc<Mutex<PrometheusState>>,
}

impl PrometheusDriver {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "prometheus-driver")]
            state: Arc::new(Mutex::new(PrometheusState::new())),
        }
    }
}

impl Default for PrometheusDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl UniLangDriver for PrometheusDriver {
    fn name(&self) -> &str {
        "prometheus"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "In-process Prometheus metrics (counters, gauges, histograms)"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Other
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "prom_counter",
            "prom_gauge",
            "prom_histogram",
            "prom_counter_inc",
            "prom_gauge_set",
            "prom_gauge_inc",
            "prom_gauge_dec",
            "prom_histogram_observe",
            "prom_export",
            "prom_serve",
        ]
    }

    #[cfg(not(feature = "prometheus-driver"))]
    fn register(&self, vm: &mut VM) {
        // Stub implementations — return descriptive errors at runtime
        let stub = |name: &'static str| {
            move |_args: &[RuntimeValue]| -> Result<RuntimeValue, RuntimeError> {
                Err(RuntimeError::type_error(format!(
                    "{}: prometheus driver requires the 'prometheus-driver' feature",
                    name
                )))
            }
        };
        vm.register_builtin("prom_counter", stub("prom_counter"));
        vm.register_builtin("prom_gauge", stub("prom_gauge"));
        vm.register_builtin("prom_histogram", stub("prom_histogram"));
        vm.register_builtin("prom_counter_inc", stub("prom_counter_inc"));
        vm.register_builtin("prom_gauge_set", stub("prom_gauge_set"));
        vm.register_builtin("prom_gauge_inc", stub("prom_gauge_inc"));
        vm.register_builtin("prom_gauge_dec", stub("prom_gauge_dec"));
        vm.register_builtin("prom_histogram_observe", stub("prom_histogram_observe"));
        vm.register_builtin("prom_export", stub("prom_export"));
        vm.register_builtin("prom_serve", stub("prom_serve"));
    }

    #[cfg(feature = "prometheus-driver")]
    fn register(&self, vm: &mut VM) {
        use prometheus::{CounterOpts, GaugeOpts, HistogramOpts};

        macro_rules! arc {
            () => {
                Arc::clone(&self.state)
            };
        }

        // prom_counter(name, help)
        {
            let state = arc!();
            vm.register_builtin("prom_counter", move |args| {
                let name = str_arg(args, 0, "prom_counter(name, help)")?;
                let help =
                    str_arg(args, 1, "prom_counter(name, help)").unwrap_or_else(|_| name.clone());
                let mut guard = state.lock().unwrap();
                if guard.metrics.contains_key(&name) {
                    return Ok(RuntimeValue::Bool(true));
                }
                let counter = Counter::with_opts(CounterOpts::new(name.clone(), help))
                    .map_err(|e| RuntimeError::type_error(format!("prom_counter: {}", e)))?;
                guard
                    .registry
                    .register(Box::new(counter.clone()))
                    .map_err(|e| {
                        RuntimeError::type_error(format!("prom_counter register: {}", e))
                    })?;
                guard.metrics.insert(name, MetricValue::Counter(counter));
                Ok(RuntimeValue::Bool(true))
            });
        }

        // prom_gauge(name, help)
        {
            let state = arc!();
            vm.register_builtin("prom_gauge", move |args| {
                let name = str_arg(args, 0, "prom_gauge(name, help)")?;
                let help =
                    str_arg(args, 1, "prom_gauge(name, help)").unwrap_or_else(|_| name.clone());
                let mut guard = state.lock().unwrap();
                if guard.metrics.contains_key(&name) {
                    return Ok(RuntimeValue::Bool(true));
                }
                let gauge = Gauge::with_opts(GaugeOpts::new(name.clone(), help))
                    .map_err(|e| RuntimeError::type_error(format!("prom_gauge: {}", e)))?;
                guard
                    .registry
                    .register(Box::new(gauge.clone()))
                    .map_err(|e| RuntimeError::type_error(format!("prom_gauge register: {}", e)))?;
                guard.metrics.insert(name, MetricValue::Gauge(gauge));
                Ok(RuntimeValue::Bool(true))
            });
        }

        // prom_histogram(name, help)
        {
            let state = arc!();
            vm.register_builtin("prom_histogram", move |args| {
                let name = str_arg(args, 0, "prom_histogram(name, help)")?;
                let help =
                    str_arg(args, 1, "prom_histogram(name, help)").unwrap_or_else(|_| name.clone());
                let mut guard = state.lock().unwrap();
                if guard.metrics.contains_key(&name) {
                    return Ok(RuntimeValue::Bool(true));
                }
                let histogram = Histogram::with_opts(HistogramOpts::new(name.clone(), help))
                    .map_err(|e| RuntimeError::type_error(format!("prom_histogram: {}", e)))?;
                guard
                    .registry
                    .register(Box::new(histogram.clone()))
                    .map_err(|e| {
                        RuntimeError::type_error(format!("prom_histogram register: {}", e))
                    })?;
                guard
                    .metrics
                    .insert(name, MetricValue::Histogram(histogram));
                Ok(RuntimeValue::Bool(true))
            });
        }

        // prom_counter_inc(name, by?)
        {
            let state = arc!();
            vm.register_builtin("prom_counter_inc", move |args| {
                let name = str_arg(args, 0, "prom_counter_inc(name, by?)")?;
                let by = float_arg(args, 1).unwrap_or(1.0);
                let guard = state.lock().unwrap();
                match guard.metrics.get(&name) {
                    Some(MetricValue::Counter(c)) => {
                        c.inc_by(by);
                        Ok(RuntimeValue::Bool(true))
                    }
                    Some(_) => Err(RuntimeError::type_error(format!(
                        "prom_counter_inc: '{}' is not a counter",
                        name
                    ))),
                    None => Err(RuntimeError::type_error(format!(
                        "prom_counter_inc: metric '{}' not found; call prom_counter() first",
                        name
                    ))),
                }
            });
        }

        // prom_gauge_set(name, value)
        {
            let state = arc!();
            vm.register_builtin("prom_gauge_set", move |args| {
                let name = str_arg(args, 0, "prom_gauge_set(name, value)")?;
                let value = float_arg(args, 1).ok_or_else(|| {
                    RuntimeError::type_error("prom_gauge_set: value must be a number".to_string())
                })?;
                let guard = state.lock().unwrap();
                match guard.metrics.get(&name) {
                    Some(MetricValue::Gauge(g)) => {
                        g.set(value);
                        Ok(RuntimeValue::Bool(true))
                    }
                    Some(_) => Err(RuntimeError::type_error(format!(
                        "prom_gauge_set: '{}' is not a gauge",
                        name
                    ))),
                    None => Err(RuntimeError::type_error(format!(
                        "prom_gauge_set: metric '{}' not found; call prom_gauge() first",
                        name
                    ))),
                }
            });
        }

        // prom_gauge_inc(name, by?)
        {
            let state = arc!();
            vm.register_builtin("prom_gauge_inc", move |args| {
                let name = str_arg(args, 0, "prom_gauge_inc(name, by?)")?;
                let by = float_arg(args, 1).unwrap_or(1.0);
                let guard = state.lock().unwrap();
                match guard.metrics.get(&name) {
                    Some(MetricValue::Gauge(g)) => {
                        g.add(by);
                        Ok(RuntimeValue::Bool(true))
                    }
                    Some(_) => Err(RuntimeError::type_error(format!(
                        "prom_gauge_inc: '{}' is not a gauge",
                        name
                    ))),
                    None => Err(RuntimeError::type_error(format!(
                        "prom_gauge_inc: metric '{}' not found",
                        name
                    ))),
                }
            });
        }

        // prom_gauge_dec(name, by?)
        {
            let state = arc!();
            vm.register_builtin("prom_gauge_dec", move |args| {
                let name = str_arg(args, 0, "prom_gauge_dec(name, by?)")?;
                let by = float_arg(args, 1).unwrap_or(1.0);
                let guard = state.lock().unwrap();
                match guard.metrics.get(&name) {
                    Some(MetricValue::Gauge(g)) => {
                        g.sub(by);
                        Ok(RuntimeValue::Bool(true))
                    }
                    Some(_) => Err(RuntimeError::type_error(format!(
                        "prom_gauge_dec: '{}' is not a gauge",
                        name
                    ))),
                    None => Err(RuntimeError::type_error(format!(
                        "prom_gauge_dec: metric '{}' not found",
                        name
                    ))),
                }
            });
        }

        // prom_histogram_observe(name, value)
        {
            let state = arc!();
            vm.register_builtin("prom_histogram_observe", move |args| {
                let name  = str_arg(args, 0, "prom_histogram_observe(name, value)")?;
                let value = float_arg(args, 1)
                    .ok_or_else(|| RuntimeError::type_error("prom_histogram_observe: value must be a number".to_string()))?;
                let guard = state.lock().unwrap();
                match guard.metrics.get(&name) {
                    Some(MetricValue::Histogram(h)) => { h.observe(value); Ok(RuntimeValue::Bool(true)) }
                    Some(_) => Err(RuntimeError::type_error(format!("prom_histogram_observe: '{}' is not a histogram", name))),
                    None => Err(RuntimeError::type_error(format!("prom_histogram_observe: metric '{}' not found; call prom_histogram() first", name))),
                }
            });
        }

        // prom_export() — return Prometheus text format
        {
            let state = arc!();
            vm.register_builtin("prom_export", move |_args| {
                let guard = state.lock().unwrap();
                let encoder = TextEncoder::new();
                let metric_families = guard.registry.gather();
                let mut buffer = Vec::new();
                encoder
                    .encode(&metric_families, &mut buffer)
                    .map_err(|e| RuntimeError::type_error(format!("prom_export: {}", e)))?;
                Ok(RuntimeValue::String(
                    String::from_utf8_lossy(&buffer).to_string(),
                ))
            });
        }

        // prom_serve(port) — start background HTTP server on /metrics
        {
            let state = arc!();
            vm.register_builtin("prom_serve", move |args| {
                let port = int_arg(args, 0).ok_or_else(|| {
                    RuntimeError::type_error("prom_serve: port must be an integer".to_string())
                })?;
                let state2 = Arc::clone(&state);
                std::thread::spawn(move || {
                    use std::io::{Read, Write};
                    use std::net::TcpListener;

                    let addr = format!("0.0.0.0:{}", port);
                    let listener = match TcpListener::bind(&addr) {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("[prom_serve] bind error on {}: {}", addr, e);
                            return;
                        }
                    };
                    println!(
                        "[prom_serve] Prometheus metrics available at http://{}:{}/metrics",
                        "0.0.0.0", port
                    );
                    for stream in listener.incoming() {
                        let stream = match stream {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        let state3 = Arc::clone(&state2);
                        std::thread::spawn(move || {
                            handle_metrics_request(stream, state3);
                        });
                    }
                });
                Ok(RuntimeValue::Bool(true))
            });
        }
    }
}

#[cfg(feature = "prometheus-driver")]
fn handle_metrics_request(mut stream: std::net::TcpStream, state: Arc<Mutex<PrometheusState>>) {
    use prometheus::{Encoder, TextEncoder};
    use std::io::{Read, Write};

    // Read the HTTP request (we don't need to parse it for a simple /metrics endpoint)
    let mut buf = [0u8; 1024];
    let _ = stream.read(&mut buf);

    let guard = state.lock().unwrap();
    let encoder = TextEncoder::new();
    let metric_families = guard.registry.gather();
    let mut body = Vec::new();
    let _ = encoder.encode(&metric_families, &mut body);
    drop(guard);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(&body);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        Some(other) => Ok(format!("{}", other)),
        None => Err(RuntimeError::type_error(format!(
            "{}: missing arg at position {}",
            sig, idx
        ))),
    }
}

fn int_arg(args: &[RuntimeValue], idx: usize) -> Option<i64> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n)) => Some(*n),
        Some(RuntimeValue::Float(f)) => Some(*f as i64),
        _ => None,
    }
}

fn float_arg(args: &[RuntimeValue], idx: usize) -> Option<f64> {
    match args.get(idx) {
        Some(RuntimeValue::Float(f)) => Some(*f),
        Some(RuntimeValue::Int(n)) => Some(*n as f64),
        _ => None,
    }
}
