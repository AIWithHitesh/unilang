// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! WebSocket server driver — accept connections and exchange text messages.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `ws_listen(host, port)` | Start WebSocket server; accepts connections in background |
//! | `ws_next_message(timeout_ms?)` | Pop next received message; returns String or Null |
//! | `ws_broadcast(message)` | Send to all connected clients; returns Int (count sent) |
//! | `ws_close()` | Stop accepting and close all connections |
//! | `ws_client_count()` | Return Int number of connected clients |

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

#[cfg(feature = "websocket")]
use tungstenite::WebSocket;

#[cfg(feature = "websocket")]
struct WsState {
    messages: Arc<Mutex<VecDeque<String>>>,
    clients: Arc<Mutex<Vec<Arc<Mutex<WebSocket<std::net::TcpStream>>>>>>,
    running: Arc<AtomicBool>,
}

pub struct WebSocketDriver {
    #[cfg(feature = "websocket")]
    state: Arc<Mutex<Option<WsState>>>,
    // These are shared Arc fields so the background threads can access them
    // even after the listen call returns.
    #[cfg(feature = "websocket")]
    messages: Arc<Mutex<VecDeque<String>>>,
    #[cfg(feature = "websocket")]
    clients: Arc<Mutex<Vec<Arc<Mutex<WebSocket<std::net::TcpStream>>>>>>,
    #[cfg(feature = "websocket")]
    running: Arc<AtomicBool>,
}

impl WebSocketDriver {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "websocket")]
            state: Arc::new(Mutex::new(None)),
            #[cfg(feature = "websocket")]
            messages: Arc::new(Mutex::new(VecDeque::new())),
            #[cfg(feature = "websocket")]
            clients: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "websocket")]
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for WebSocketDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl UniLangDriver for WebSocketDriver {
    fn name(&self) -> &str {
        "websocket"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "WebSocket server (accept connections, broadcast messages)"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Other
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &[
            "ws_listen",
            "ws_next_message",
            "ws_broadcast",
            "ws_close",
            "ws_client_count",
        ]
    }

    #[cfg(not(feature = "websocket"))]
    fn register(&self, vm: &mut VM) {
        // Stub implementations — return descriptive errors at runtime
        let stub = |name: &'static str| {
            move |_args: &[RuntimeValue]| -> Result<RuntimeValue, RuntimeError> {
                Err(RuntimeError::type_error(format!(
                    "{}: websocket driver requires the 'websocket' feature",
                    name
                )))
            }
        };
        vm.register_builtin("ws_listen", stub("ws_listen"));
        vm.register_builtin("ws_next_message", stub("ws_next_message"));
        vm.register_builtin("ws_broadcast", stub("ws_broadcast"));
        vm.register_builtin("ws_close", stub("ws_close"));
        vm.register_builtin("ws_client_count", stub("ws_client_count"));
    }

    #[cfg(feature = "websocket")]
    fn register(&self, vm: &mut VM) {
        use std::net::TcpListener;
        use tungstenite::Message;

        let messages = Arc::clone(&self.messages);
        let clients = Arc::clone(&self.clients);
        let running = Arc::clone(&self.running);

        // ws_listen(host, port)
        {
            let messages2 = Arc::clone(&messages);
            let clients2 = Arc::clone(&clients);
            let running2 = Arc::clone(&running);
            vm.register_builtin("ws_listen", move |args| {
                let host = str_arg(args, 0, "ws_listen(host, port)")?;
                let port = int_arg(args, 1).ok_or_else(|| {
                    RuntimeError::type_error("ws_listen: port must be an integer".to_string())
                })?;
                let addr = format!("{}:{}", host, port);

                let listener = TcpListener::bind(&addr).map_err(|e| {
                    RuntimeError::type_error(format!("ws_listen bind {}: {}", addr, e))
                })?;
                // Enable non-blocking so the accept loop can check the running flag
                listener
                    .set_nonblocking(false)
                    .map_err(|e| RuntimeError::type_error(format!("ws_listen: {}", e)))?;

                running2.store(true, Ordering::SeqCst);
                println!("[ws_listen] WebSocket server listening on ws://{}", addr);

                let messages3 = Arc::clone(&messages2);
                let clients3 = Arc::clone(&clients2);
                let running3 = Arc::clone(&running2);

                std::thread::spawn(move || {
                    listener.set_nonblocking(true).ok();
                    while running3.load(Ordering::SeqCst) {
                        match listener.accept() {
                            Ok((tcp_stream, peer_addr)) => {
                                println!("[ws_listen] new connection from {}", peer_addr);
                                let messages4 = Arc::clone(&messages3);
                                let clients4 = Arc::clone(&clients3);

                                std::thread::spawn(move || {
                                    let ws = match tungstenite::accept(tcp_stream) {
                                        Ok(ws) => ws,
                                        Err(e) => {
                                            eprintln!("[ws_listen] handshake error: {}", e);
                                            return;
                                        }
                                    };
                                    let ws_arc = Arc::new(Mutex::new(ws));
                                    clients4.lock().unwrap().push(Arc::clone(&ws_arc));

                                    loop {
                                        let msg_result = {
                                            let mut guard = ws_arc.lock().unwrap();
                                            guard.read_message()
                                        };
                                        match msg_result {
                                            Ok(Message::Text(text)) => {
                                                messages4.lock().unwrap().push_back(text);
                                            }
                                            Ok(Message::Binary(data)) => {
                                                let text =
                                                    String::from_utf8_lossy(&data).to_string();
                                                messages4.lock().unwrap().push_back(text);
                                            }
                                            Ok(Message::Close(_)) | Err(_) => break,
                                            Ok(_) => {} // ping/pong/etc
                                        }
                                    }
                                    // Remove closed client from the list
                                    // (best-effort; failed sends will be silently skipped in ws_broadcast)
                                });
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                            Err(e) => {
                                eprintln!("[ws_listen] accept error: {}", e);
                            }
                        }
                    }
                });

                Ok(RuntimeValue::Bool(true))
            });
        }

        // ws_next_message(timeout_ms?)
        {
            let messages2 = Arc::clone(&messages);
            vm.register_builtin("ws_next_message", move |args| {
                let timeout_ms = int_arg(args, 0).unwrap_or(0) as u64;
                let deadline =
                    std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
                loop {
                    if let Some(msg) = messages2.lock().unwrap().pop_front() {
                        return Ok(RuntimeValue::String(msg));
                    }
                    if timeout_ms == 0 || std::time::Instant::now() >= deadline {
                        return Ok(RuntimeValue::Null);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            });
        }

        // ws_broadcast(message)
        {
            let clients2 = Arc::clone(&clients);
            vm.register_builtin("ws_broadcast", move |args| {
                use tungstenite::Message;
                let message = str_arg(args, 0, "ws_broadcast(message)")?;
                let guard = clients2.lock().unwrap();
                let mut sent = 0i64;
                for ws_arc in guard.iter() {
                    let mut ws = ws_arc.lock().unwrap();
                    if ws.write_message(Message::Text(message.clone())).is_ok() {
                        sent += 1;
                    }
                }
                Ok(RuntimeValue::Int(sent))
            });
        }

        // ws_close()
        {
            let running2 = Arc::clone(&running);
            let clients2 = Arc::clone(&clients);
            vm.register_builtin("ws_close", move |_args| {
                use tungstenite::Message;
                running2.store(false, Ordering::SeqCst);
                let guard = clients2.lock().unwrap();
                for ws_arc in guard.iter() {
                    let mut ws = ws_arc.lock().unwrap();
                    let _ = ws.close(None);
                }
                Ok(RuntimeValue::Bool(true))
            });
        }

        // ws_client_count()
        {
            let clients2 = Arc::clone(&clients);
            vm.register_builtin("ws_client_count", move |_args| {
                let count = clients2.lock().unwrap().len() as i64;
                Ok(RuntimeValue::Int(count))
            });
        }
    }
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
