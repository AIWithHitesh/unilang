// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.

//! `unilang-jupyter` — Jupyter kernel for UniLang.
//!
//! Implements the [Jupyter Messaging Protocol v5.3](https://jupyter-client.readthedocs.io/en/stable/messaging.html)
//! over ZMQ.  The kernel binary is launched by Jupyter when the user selects the
//! "UniLang" kernel; Jupyter passes a connection-file JSON that contains the TCP
//! ports and HMAC key to use.
//!
//! Wire-protocol message layout (multi-part ZMQ frames):
//! ```text
//!   [identity...]  "<IDS|MSG>"  hmac  header  parent_header  metadata  content  [buffers...]
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use uuid::Uuid;
use zeromq::{DealerSocket, PubSocket, RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

// ── Connection file ───────────────────────────────────────────────────────────

/// Deserialised representation of the Jupyter connection file.
#[derive(serde::Deserialize, Debug)]
struct ConnectionInfo {
    ip: String,
    transport: String,
    shell_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    control_port: u16,
    hb_port: u16,
    key: String,
    signature_scheme: String,
}

impl ConnectionInfo {
    fn endpoint(&self, port: u16) -> String {
        format!("{}://{}:{}", self.transport, self.ip, port)
    }
}

// ── HMAC signing ─────────────────────────────────────────────────────────────

/// Compute the HMAC-SHA256 signature required by the Jupyter wire protocol.
fn sign(key: &str, parts: &[&[u8]]) -> String {
    if key.is_empty() {
        return String::new();
    }
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length");
    for part in parts {
        mac.update(part);
    }
    hex::encode(mac.finalize().into_bytes())
}

// ── Message building helpers ──────────────────────────────────────────────────

/// Build a Jupyter header JSON blob.
fn make_header(msg_type: &str, session: &str) -> Value {
    json!({
        "msg_id":   Uuid::new_v4().to_string(),
        "session":  session,
        "username": "kernel",
        "date":     Utc::now().to_rfc3339(),
        "msg_type": msg_type,
        "version":  "5.3"
    })
}

/// Serialise a JSON value to compact bytes.
fn to_bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).expect("JSON serialisation is infallible for serde_json::Value")
}

/// Pack a Jupyter message into a `ZmqMessage` with zero or more identity frames.
///
/// Frame layout:
/// ```text
/// [ident_0] ... [ident_n] <IDS|MSG> hmac header parent_header metadata content
/// ```
fn build_message(
    key: &str,
    session: &str,
    identities: &[Vec<u8>],
    parent_header: &Value,
    msg_type: &str,
    content: Value,
) -> ZmqMessage {
    let header = make_header(msg_type, session);
    let metadata = json!({});

    let h_bytes = to_bytes(&header);
    let ph_bytes = to_bytes(parent_header);
    let m_bytes = to_bytes(&metadata);
    let c_bytes = to_bytes(&content);

    let sig = sign(key, &[&h_bytes, &ph_bytes, &m_bytes, &c_bytes]);

    let mut frames: Vec<bytes::Bytes> = Vec::new();
    for id in identities {
        frames.push(bytes::Bytes::from(id.clone()));
    }
    frames.push(bytes::Bytes::from_static(b"<IDS|MSG>"));
    frames.push(bytes::Bytes::from(sig.into_bytes()));
    frames.push(bytes::Bytes::from(h_bytes));
    frames.push(bytes::Bytes::from(ph_bytes));
    frames.push(bytes::Bytes::from(m_bytes));
    frames.push(bytes::Bytes::from(c_bytes));

    ZmqMessage::try_from(frames).expect("build_message: frames must not be empty")
}

// ── Message parsing ───────────────────────────────────────────────────────────

/// A parsed incoming Jupyter message.
struct IncomingMsg {
    identities: Vec<Vec<u8>>,
    header: Value,
    #[allow(dead_code)]
    parent_header: Value,
    #[allow(dead_code)]
    metadata: Value,
    content: Value,
}

impl IncomingMsg {
    fn msg_type(&self) -> &str {
        self.header["msg_type"].as_str().unwrap_or("")
    }
}

/// Parse a raw multi-frame ZMQ message into an `IncomingMsg`.
fn parse_message(msg: ZmqMessage) -> Option<IncomingMsg> {
    let frames: Vec<bytes::Bytes> = msg.into_vec();

    // Locate the <IDS|MSG> delimiter.
    let delim_pos = frames.iter().position(|f| f.as_ref() == b"<IDS|MSG>")?;

    let identities: Vec<Vec<u8>> = frames[..delim_pos].iter().map(|f| f.to_vec()).collect();

    // After delimiter: hmac, header, parent_header, metadata, content
    let after = &frames[delim_pos + 1..];
    if after.len() < 5 {
        return None;
    }

    let header = serde_json::from_slice(&after[1]).ok()?;
    let parent_header = serde_json::from_slice(&after[2]).ok()?;
    let metadata = serde_json::from_slice(&after[3]).ok()?;
    let content = serde_json::from_slice(&after[4]).ok()?;

    Some(IncomingMsg {
        identities,
        header,
        parent_header,
        metadata,
        content,
    })
}

// ── UniLang VM execution ──────────────────────────────────────────────────────

/// Execute a snippet of UniLang source code through the full VM pipeline.
///
/// Returns `(stdout_lines, error_message)`.  Any captured print output is
/// returned as `stdout_lines`; if execution fails the error message is in
/// `error_message`.
fn execute_unilang(code: &str) -> (Vec<String>, Option<String>) {
    use unilang_common::source::SourceMap;
    use unilang_runtime::error::ErrorKind;

    let mut source_map = SourceMap::new();
    let source_id = source_map.add("<jupyter>".to_string(), code.to_string());

    // ── Parse ─────────────────────────────────────────────────────────────────
    let (module, parse_diags) = unilang_parser::parse(source_id, code);
    if parse_diags.has_errors() {
        let msgs: Vec<String> = parse_diags
            .diagnostics()
            .iter()
            .map(|d| d.message.clone())
            .collect();
        return (vec![], Some(format!("ParseError: {}", msgs.join("; "))));
    }

    // ── Semantic analysis ─────────────────────────────────────────────────────
    let driver_funcs = unilang_drivers::default_registry().all_function_names();
    let (_result, sem_diags) =
        unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);
    if sem_diags.has_errors() {
        let msgs: Vec<String> = sem_diags
            .diagnostics()
            .iter()
            .map(|d| d.message.clone())
            .collect();
        return (vec![], Some(format!("SemanticError: {}", msgs.join("; "))));
    }

    // ── Codegen ───────────────────────────────────────────────────────────────
    let bytecode = match unilang_codegen::compile(&module) {
        Ok(bc) => bc,
        Err(diags) => {
            let msgs: Vec<String> = diags.iter().map(|d| d.message.clone()).collect();
            return (vec![], Some(format!("CodegenError: {}", msgs.join("; "))));
        }
    };

    // ── VM execution with output capture ─────────────────────────────────────
    // Use `new_with_capture()` so that print() calls are collected into the
    // VM's internal output buffer rather than written directly to stdout.
    let mut vm = unilang_runtime::vm::VM::new_with_capture();
    unilang_stdlib::register_builtins(&mut vm);
    let drivers = unilang_drivers::default_registry();
    drivers.register_all(&mut vm);

    match vm.run(&bytecode) {
        Ok(_) => {
            let captured: Vec<String> = vm.output().to_vec();
            (captured, None)
        }
        Err(e) => {
            // ErrorKind::Halt is a normal program termination, not an error.
            let captured: Vec<String> = vm.output().to_vec();
            if e.kind == ErrorKind::Halt {
                (captured, None)
            } else {
                (captured, Some(e.message.clone()))
            }
        }
    }
}

// ── Status helpers ────────────────────────────────────────────────────────────

/// Send an `iopub` status message (`busy` or `idle`).
async fn send_status(
    iopub: &mut PubSocket,
    key: &str,
    session: &str,
    parent_header: &Value,
    status: &str,
) {
    let msg = build_message(
        key,
        session,
        &[],
        parent_header,
        "status",
        json!({ "execution_state": status }),
    );
    let _ = iopub.send(msg).await;
}

// ── Shell handler ─────────────────────────────────────────────────────────────

/// Handle a single shell-channel message and produce replies on `shell` and `iopub`.
async fn handle_shell(
    shell: &mut RouterSocket,
    iopub: &mut PubSocket,
    key: &str,
    session: &str,
    exec_count: &Arc<AtomicU64>,
    msg: IncomingMsg,
) {
    match msg.msg_type() {
        // ── kernel_info_request ───────────────────────────────────────────────
        "kernel_info_request" => {
            send_status(iopub, key, session, &msg.header, "busy").await;

            let reply_content = json!({
                "status":           "ok",
                "protocol_version": "5.3",
                "implementation":   "unilang-jupyter",
                "implementation_version": env!("CARGO_PKG_VERSION"),
                "language_info": {
                    "name":           "unilang",
                    "version":        "0.1.0",
                    "mimetype":       "text/x-unilang",
                    "file_extension": ".uniL"
                },
                "banner": "UniLang Jupyter Kernel",
                "help_links": []
            });

            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "kernel_info_reply",
                reply_content,
            );
            let _ = shell.send(reply).await;

            send_status(iopub, key, session, &msg.header, "idle").await;
        }

        // ── execute_request ───────────────────────────────────────────────────
        "execute_request" => {
            send_status(iopub, key, session, &msg.header, "busy").await;

            let code = msg.content["code"].as_str().unwrap_or("").to_string();
            let silent = msg.content["silent"].as_bool().unwrap_or(false);
            let store_history = msg.content["store_history"].as_bool().unwrap_or(true);

            let exec_no = if !silent && store_history {
                exec_count.fetch_add(1, Ordering::SeqCst) + 1
            } else {
                exec_count.load(Ordering::SeqCst)
            };

            // Broadcast the input back to iopub so notebooks can display it.
            if !silent {
                let input_msg = build_message(
                    key,
                    session,
                    &[],
                    &msg.header,
                    "execute_input",
                    json!({
                        "code":            code,
                        "execution_count": exec_no
                    }),
                );
                let _ = iopub.send(input_msg).await;
            }

            // Run through the UniLang VM.
            let (stdout_lines, error) = execute_unilang(&code);

            // Stream stdout lines.
            if !silent && !stdout_lines.is_empty() {
                let text = stdout_lines.join("\n") + "\n";
                let stream_msg = build_message(
                    key,
                    session,
                    &[],
                    &msg.header,
                    "stream",
                    json!({ "name": "stdout", "text": text }),
                );
                let _ = iopub.send(stream_msg).await;
            }

            let (reply_content, status) = if let Some(err_msg) = &error {
                // Send an error message on iopub.
                let ename = extract_ename(err_msg);
                let evalue = err_msg.clone();
                let traceback = vec![evalue.clone()];

                if !silent {
                    let err_pub = build_message(
                        key,
                        session,
                        &[],
                        &msg.header,
                        "error",
                        json!({
                            "ename":     ename,
                            "evalue":    evalue,
                            "traceback": traceback
                        }),
                    );
                    let _ = iopub.send(err_pub).await;
                }

                (
                    json!({
                        "status":           "error",
                        "execution_count":  exec_no,
                        "ename":            ename,
                        "evalue":           evalue,
                        "traceback":        [evalue]
                    }),
                    "error",
                )
            } else {
                (
                    json!({
                        "status":           "ok",
                        "execution_count":  exec_no,
                        "payload":          [],
                        "user_expressions": {}
                    }),
                    "ok",
                )
            };

            // If execution succeeded and produced output that should be shown
            // as a result (no explicit print), that was already sent as stream.
            // We send execute_result only when status is ok and there is a
            // meaningful last-expression value — the VM captures all print
            // calls, so for now we only echo non-empty output that wasn't
            // already streamed.
            if !silent && status == "ok" && !stdout_lines.is_empty() {
                // Already sent as stream — nothing extra to emit here.
                // If desired, a future version can return the last
                // expression's value as execute_result instead.
            }

            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "execute_reply",
                reply_content,
            );
            let _ = shell.send(reply).await;

            send_status(iopub, key, session, &msg.header, "idle").await;
        }

        // ── comm_info_request ─────────────────────────────────────────────────
        "comm_info_request" => {
            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "comm_info_reply",
                json!({ "status": "ok", "comms": {} }),
            );
            let _ = shell.send(reply).await;
        }

        // ── is_complete_request ───────────────────────────────────────────────
        "is_complete_request" => {
            // Optimistic: always claim the code is complete.
            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "is_complete_reply",
                json!({ "status": "complete" }),
            );
            let _ = shell.send(reply).await;
        }

        // ── complete_request ──────────────────────────────────────────────────
        "complete_request" => {
            // Stub — return empty completions.
            let cursor_pos = msg.content["cursor_pos"].as_u64().unwrap_or(0);
            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "complete_reply",
                json!({
                    "status":       "ok",
                    "matches":      [],
                    "cursor_start": cursor_pos,
                    "cursor_end":   cursor_pos,
                    "metadata":     {}
                }),
            );
            let _ = shell.send(reply).await;
        }

        // ── inspect_request ───────────────────────────────────────────────────
        "inspect_request" => {
            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "inspect_reply",
                json!({ "status": "ok", "found": false, "data": {}, "metadata": {} }),
            );
            let _ = shell.send(reply).await;
        }

        // ── shutdown_request ──────────────────────────────────────────────────
        "shutdown_request" => {
            let restart = msg.content["restart"].as_bool().unwrap_or(false);
            let reply = build_message(
                key,
                session,
                &msg.identities,
                &msg.header,
                "shutdown_reply",
                json!({ "status": "ok", "restart": restart }),
            );
            let _ = shell.send(reply).await;
            // The event loop will exit on its own once Jupyter closes sockets.
        }

        other => {
            eprintln!("[unilang-jupyter] unhandled shell msg_type: {}", other);
        }
    }
}

// ── Utility ───────────────────────────────────────────────────────────────────

/// Extract an error-name token from an error message string, e.g.
/// `"ParseError: ..."` → `"ParseError"`.
fn extract_ename(msg: &str) -> &str {
    if let Some(pos) = msg.find(':') {
        let candidate = msg[..pos].trim();
        if !candidate.contains(' ') {
            return candidate;
        }
    }
    "RuntimeError"
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Locate --connection-file argument.
    let conn_file = args
        .iter()
        .position(|a| a == "--connection-file")
        .and_then(|i| args.get(i + 1))
        .expect("Usage: unilang-jupyter --connection-file <path>");

    let raw = std::fs::read_to_string(conn_file)
        .unwrap_or_else(|e| panic!("Cannot read connection file '{}': {}", conn_file, e));

    let conn: ConnectionInfo = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("Invalid connection file JSON: {}", e));

    if conn.signature_scheme != "hmac-sha256" && !conn.key.is_empty() {
        eprintln!(
            "[unilang-jupyter] Warning: unsupported signature_scheme '{}', expected hmac-sha256",
            conn.signature_scheme
        );
    }

    let key = conn.key.clone();
    let session = Uuid::new_v4().to_string();

    // ── Bind ZMQ sockets ──────────────────────────────────────────────────────
    let mut shell_sock: RouterSocket = zeromq::RouterSocket::new();
    shell_sock
        .bind(&conn.endpoint(conn.shell_port))
        .await
        .unwrap_or_else(|e| panic!("Cannot bind shell socket: {}", e));

    let mut iopub_sock: PubSocket = zeromq::PubSocket::new();
    iopub_sock
        .bind(&conn.endpoint(conn.iopub_port))
        .await
        .unwrap_or_else(|e| panic!("Cannot bind iopub socket: {}", e));

    let mut control_sock: RouterSocket = zeromq::RouterSocket::new();
    control_sock
        .bind(&conn.endpoint(conn.control_port))
        .await
        .unwrap_or_else(|e| panic!("Cannot bind control socket: {}", e));

    let mut hb_sock: zeromq::RepSocket = zeromq::RepSocket::new();
    hb_sock
        .bind(&conn.endpoint(conn.hb_port))
        .await
        .unwrap_or_else(|e| panic!("Cannot bind heartbeat socket: {}", e));

    // stdin socket — bind but mostly ignore (kernel does not request input).
    // The socket must still be bound so Jupyter does not complain about an
    // unresponsive port.
    let mut _stdin_sock: DealerSocket = zeromq::DealerSocket::new();
    _stdin_sock
        .bind(&conn.endpoint(conn.stdin_port))
        .await
        .unwrap_or_else(|e| panic!("Cannot bind stdin socket: {}", e));

    let exec_count = Arc::new(AtomicU64::new(0));

    // Announce kernel as idle.
    let _ = send_status_standalone(&mut iopub_sock, &key, &session, "idle").await;

    eprintln!("[unilang-jupyter] kernel ready — session {}", session);

    // ── Main event loop ───────────────────────────────────────────────────────
    loop {
        tokio::select! {
            // Heartbeat — echo back whatever we receive.
            hb_result = hb_sock.recv() => {
                if let Ok(msg) = hb_result {
                    let _ = hb_sock.send(msg).await;
                }
            }

            // Shell channel — main execution requests.
            shell_result = shell_sock.recv() => {
                if let Ok(raw_msg) = shell_result {
                    if let Some(parsed) = parse_message(raw_msg) {
                        handle_shell(
                            &mut shell_sock,
                            &mut iopub_sock,
                            &key,
                            &session,
                            &exec_count,
                            parsed,
                        )
                        .await;
                    }
                }
            }

            // Control channel — shutdown etc.  We reuse the shell handler
            // since the message types are the same.
            ctrl_result = control_sock.recv() => {
                if let Ok(raw_msg) = ctrl_result {
                    if let Some(parsed) = parse_message(raw_msg) {
                        let msg_type = parsed.msg_type().to_string();
                        handle_shell(
                            &mut control_sock,
                            &mut iopub_sock,
                            &key,
                            &session,
                            &exec_count,
                            parsed,
                        )
                        .await;
                        if msg_type == "shutdown_request" {
                            eprintln!("[unilang-jupyter] shutdown requested, exiting.");
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Helper — send a status message without an incoming parent header.
async fn send_status_standalone(iopub: &mut PubSocket, key: &str, session: &str, status: &str) {
    let empty_parent = json!({});
    let msg = build_message(
        key,
        session,
        &[],
        &empty_parent,
        "status",
        json!({ "execution_state": status }),
    );
    let _ = iopub.send(msg).await;
}
