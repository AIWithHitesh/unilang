// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! SMTP Email driver — minimal synchronous SMTP/STARTTLS client.
//!
//! Implemented over `std::net::TcpStream` + `rustls` (TLS).
//! Supports STARTTLS on port 587 (default) and implicit TLS (SMTPS) on port 465.
//! Authentication: AUTH PLAIN.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `smtp_connect(host, port, username, password)` | Store SMTP connection config |
//! | `smtp_send(from, to, subject, body)` | Send plain-text email (`to` can be String or List) |
//! | `smtp_send_html(from, to, subject, html_body)` | Send HTML email |

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use rustls::ClientConnection;
use rustls_pki_types::ServerName;

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct SmtpState {
    host: String,
    port: u16,
    username: String,
    password: String,
}

pub struct SmtpDriver {
    state: Arc<Mutex<Option<SmtpState>>>,
}

impl SmtpDriver {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for SmtpDriver {
    fn default() -> Self {
        Self::new()
    }
}

// ── Trait impl ───────────────────────────────────────────────────────────────

impl UniLangDriver for SmtpDriver {
    fn name(&self) -> &str {
        "smtp"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "SMTP email delivery (plain-text and HTML) via STARTTLS/TLS using rustls"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Other
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &["smtp_connect", "smtp_send", "smtp_send_html"]
    }

    fn register(&self, vm: &mut VM) {
        macro_rules! arc {
            () => {
                Arc::clone(&self.state)
            };
        }

        // smtp_connect(host, port, username, password)
        {
            let state = arc!();
            vm.register_builtin("smtp_connect", move |args| {
                let host = str_arg(args, 0, "smtp_connect(host, port, username, password)")?;
                let port = int_arg(args, 1).unwrap_or(587) as u16;
                let username = str_arg(args, 2, "smtp_connect(host, port, username, password)")?;
                let password = str_arg(args, 3, "smtp_connect(host, port, username, password)")?;
                *state.lock().unwrap() = Some(SmtpState {
                    host,
                    port,
                    username,
                    password,
                });
                Ok(RuntimeValue::Bool(true))
            });
        }

        // smtp_send(from, to, subject, body)
        {
            let state = arc!();
            vm.register_builtin("smtp_send", move |args| {
                let from = str_arg(args, 0, "smtp_send(from, to, subject, body)")?;
                let to_list = extract_recipients(args, 1, "smtp_send")?;
                let subject = str_arg(args, 2, "smtp_send(from, to, subject, body)")?;
                let body = str_arg(args, 3, "smtp_send(from, to, subject, body)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("smtp_send"))?;
                let msg = build_message(&from, &to_list, &subject, &body, "text/plain");
                send_email(cfg, &from, &to_list, &msg)
                    .map_err(|e| RuntimeError::type_error(format!("smtp_send: {}", e)))?;
                Ok(RuntimeValue::Bool(true))
            });
        }

        // smtp_send_html(from, to, subject, html_body)
        {
            let state = arc!();
            vm.register_builtin("smtp_send_html", move |args| {
                let from = str_arg(args, 0, "smtp_send_html(from, to, subject, html_body)")?;
                let to_list = extract_recipients(args, 1, "smtp_send_html")?;
                let subject = str_arg(args, 2, "smtp_send_html(from, to, subject, html_body)")?;
                let html_body = str_arg(args, 3, "smtp_send_html(from, to, subject, html_body)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("smtp_send_html"))?;
                let msg = build_message(&from, &to_list, &subject, &html_body, "text/html");
                send_email(cfg, &from, &to_list, &msg)
                    .map_err(|e| RuntimeError::type_error(format!("smtp_send_html: {}", e)))?;
                Ok(RuntimeValue::Bool(true))
            });
        }
    }
}

// ── SMTP session ──────────────────────────────────────────────────────────────

fn send_email(
    cfg: &SmtpState,
    from: &str,
    recipients: &[String],
    raw_message: &str,
) -> Result<(), String> {
    let addr = format!("{}:{}", cfg.host, cfg.port);
    let tcp = TcpStream::connect(&addr).map_err(|e| format!("connect to {}: {}", addr, e))?;
    tcp.set_read_timeout(Some(Duration::from_secs(30))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(30))).ok();

    if cfg.port == 465 {
        // Implicit TLS (SMTPS)
        let mut tls = upgrade_to_tls(tcp, &cfg.host)?;
        smtp_session(
            &mut tls,
            from,
            recipients,
            raw_message,
            &cfg.username,
            &cfg.password,
        )
    } else {
        // STARTTLS: plain handshake first, then TLS upgrade
        let read_tcp = tcp.try_clone().map_err(|e| e.to_string())?;
        let mut write_tcp = tcp;
        let mut reader = BufReader::new(read_tcp);

        smtp_read_response(&mut reader, 220)?;
        smtp_write(&mut write_tcp, "EHLO smtp-client")?;
        smtp_read_multiline(&mut reader, 250)?;
        smtp_write(&mut write_tcp, "STARTTLS")?;
        smtp_read_response(&mut reader, 220)?;
        drop(reader);

        let mut tls = upgrade_to_tls(write_tcp, &cfg.host)?;
        smtp_session(
            &mut tls,
            from,
            recipients,
            raw_message,
            &cfg.username,
            &cfg.password,
        )
    }
}

/// Run the authenticated part of the SMTP dialogue over any Read+Write stream.
fn smtp_session<RW: std::io::Read + std::io::Write>(
    rw: &mut RW,
    from: &str,
    recipients: &[String],
    raw_message: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    // After TLS: EHLO again
    smtp_write(rw, "EHLO smtp-client")?;
    smtp_read_multiline_rw(rw, 250)?;

    // AUTH PLAIN: base64("\x00username\x00password")
    let auth_b64 = base64::engine::general_purpose::STANDARD
        .encode(format!("\x00{}\x00{}", username, password));
    smtp_write(rw, &format!("AUTH PLAIN {}", auth_b64))?;
    smtp_read_response_rw(rw, 235)?;

    // MAIL FROM
    smtp_write(rw, &format!("MAIL FROM:<{}>", from))?;
    smtp_read_response_rw(rw, 250)?;

    // RCPT TO
    for recipient in recipients {
        smtp_write(rw, &format!("RCPT TO:<{}>", recipient))?;
        smtp_read_response_rw(rw, 250)?;
    }

    // DATA
    smtp_write(rw, "DATA")?;
    smtp_read_response_rw(rw, 354)?;

    // Message body (dot-stuffed) followed by "."
    let stuffed = dot_stuff(raw_message);
    rw.write_all(stuffed.as_bytes())
        .map_err(|e| e.to_string())?;
    smtp_write(rw, ".")?;
    smtp_read_response_rw(rw, 250)?;

    // QUIT
    smtp_write(rw, "QUIT")?;
    // ignore the 221 response
    let _ = smtp_read_response_rw(rw, 221);
    Ok(())
}

// ── TLS helpers ───────────────────────────────────────────────────────────────

fn build_tls_config() -> Arc<rustls::ClientConfig> {
    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
    };
    Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    )
}

fn upgrade_to_tls(
    tcp: TcpStream,
    host: &str,
) -> Result<rustls::StreamOwned<ClientConnection, TcpStream>, String> {
    let config = build_tls_config();
    let server_name = ServerName::try_from(host.to_string())
        .map_err(|e| format!("invalid TLS server name '{}': {}", host, e))?;
    let conn = ClientConnection::new(config, server_name)
        .map_err(|e| format!("TLS handshake init: {}", e))?;
    Ok(rustls::StreamOwned::new(conn, tcp))
}

// ── Low-level SMTP I/O ────────────────────────────────────────────────────────

fn smtp_write<W: Write>(w: &mut W, cmd: &str) -> Result<(), String> {
    write!(w, "{}\r\n", cmd).map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())
}

/// Read a (potentially multi-line) SMTP response from a `BufRead` source.
fn smtp_read_multiline<R: BufRead>(r: &mut R, expected: u32) -> Result<String, String> {
    let mut last_text = String::new();
    loop {
        let mut line = String::new();
        r.read_line(&mut line).map_err(|e| e.to_string())?;
        if line.len() < 4 {
            return Err(format!("short SMTP response line: {:?}", line));
        }
        let code: u32 = line[..3]
            .parse()
            .map_err(|_| format!("malformed SMTP code in: {:?}", line))?;
        if code != expected {
            return Err(format!(
                "expected {}, got {} ({})",
                expected,
                code,
                line.trim_end()
            ));
        }
        last_text = line[4..].trim_end().to_string();
        if line.chars().nth(3) == Some(' ') {
            break;
        }
    }
    Ok(last_text)
}

fn smtp_read_response<R: BufRead>(r: &mut R, expected: u32) -> Result<String, String> {
    smtp_read_multiline(r, expected)
}

/// Read from a `Read` source (wraps in a temporary BufReader).
fn smtp_read_multiline_rw<R: std::io::Read>(r: &mut R, expected: u32) -> Result<String, String> {
    let mut br = BufReader::new(r.by_ref());
    smtp_read_multiline(&mut br, expected)
}

fn smtp_read_response_rw<R: std::io::Read>(r: &mut R, expected: u32) -> Result<String, String> {
    smtp_read_multiline_rw(r, expected)
}

// ── MIME message builders ─────────────────────────────────────────────────────

fn build_message(
    from: &str,
    to: &[String],
    subject: &str,
    body: &str,
    content_type: &str,
) -> String {
    format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\n\
         Content-Type: {}; charset=UTF-8\r\nContent-Transfer-Encoding: 7bit\r\n\r\n{}\r\n",
        from,
        to.join(", "),
        subject,
        content_type,
        body,
    )
}

/// RFC 5321 dot-stuffing: prefix any line that starts with '.' with another '.'.
fn dot_stuff(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len() + 16);
    for line in msg.lines() {
        if line.starts_with('.') {
            out.push('.');
        }
        out.push_str(line);
        out.push_str("\r\n");
    }
    out
}

// ── Common arg helpers ────────────────────────────────────────────────────────

fn extract_recipients(
    args: &[RuntimeValue],
    idx: usize,
    func: &str,
) -> Result<Vec<String>, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(vec![s.clone()]),
        Some(RuntimeValue::List(items)) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    RuntimeValue::String(s) => out.push(s.clone()),
                    other => {
                        return Err(RuntimeError::type_error(format!(
                            "{}: 'to' list items must be strings, got {:?}",
                            func, other
                        )))
                    }
                }
            }
            Ok(out)
        }
        _ => Err(RuntimeError::type_error(format!(
            "{}: 'to' must be a String or List of Strings",
            func
        ))),
    }
}

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call smtp_connect() first", func))
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

fn int_arg(args: &[RuntimeValue], idx: usize) -> Option<i64> {
    match args.get(idx) {
        Some(RuntimeValue::Int(n)) => Some(*n),
        Some(RuntimeValue::Float(f)) => Some(*f as i64),
        _ => None,
    }
}
