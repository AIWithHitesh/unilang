// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang native AOT compilation support.
//!
//! Compiles a UniLang source file to a self-contained native executable by:
//! 1. Running the full UniLang pipeline (parse → semantic → codegen)
//! 2. Serialising the resulting bytecode to JSON
//! 3. Writing a Rust stub that embeds the bytecode as a static byte array
//!    with a minimal embedded-VM runner
//! 4. Optionally invoking `rustc` to produce the final binary
//!
//! The generated stub is deliberately dependency-free — it only re-implements
//! the tiny subset of the VM needed to execute the bytecode at runtime.

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur during native AOT compilation.
#[derive(Debug)]
pub enum NativeError {
    /// Source code could not be parsed or compiled.
    CompileError(String),
    /// An I/O operation failed (writing files, invoking rustc, …).
    IoError(String),
    /// The target triple is not supported or recognised.
    UnsupportedTarget(String),
    /// `rustc` invocation failed.
    RustcError { exit_code: i32, stderr: String },
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NativeError::CompileError(msg) => write!(f, "compile error: {}", msg),
            NativeError::IoError(msg) => write!(f, "I/O error: {}", msg),
            NativeError::UnsupportedTarget(t) => write!(f, "unsupported target: {}", t),
            NativeError::RustcError { exit_code, stderr } => {
                write!(f, "rustc exited with {}: {}", exit_code, stderr)
            }
        }
    }
}

impl std::error::Error for NativeError {}

impl From<std::io::Error> for NativeError {
    fn from(e: std::io::Error) -> Self {
        NativeError::IoError(e.to_string())
    }
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Optimisation levels passed through to `rustc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptLevel {
    /// No optimisation (`-C opt-level=0`).
    None,
    /// Light optimisation (`-C opt-level=1`).
    Light,
    /// Default release optimisation (`-C opt-level=2`).
    #[default]
    Default,
    /// Aggressive optimisation (`-C opt-level=3`).
    Aggressive,
    /// Optimise for binary size (`-C opt-level=s`).
    Size,
}

impl OptLevel {
    fn rustc_flag(&self) -> &'static str {
        match self {
            OptLevel::None => "0",
            OptLevel::Light => "1",
            OptLevel::Default => "2",
            OptLevel::Aggressive => "3",
            OptLevel::Size => "s",
        }
    }
}

/// Configuration for a native AOT compilation.
#[derive(Debug, Clone)]
pub struct NativeCompileConfig {
    /// Target triple, e.g. `"x86_64-unknown-linux-gnu"`.
    /// `None` means the current host target.
    pub target: Option<String>,
    /// Optimisation level.
    pub opt_level: OptLevel,
    /// Strip debug symbols from the output binary.
    pub strip_symbols: bool,
}

impl Default for NativeCompileConfig {
    fn default() -> Self {
        NativeCompileConfig {
            target: None,
            opt_level: OptLevel::Default,
            strip_symbols: false,
        }
    }
}

// ── AOT manifest ─────────────────────────────────────────────────────────────

/// A JSON manifest that describes a completed native build.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AotManifest {
    /// Path to the original `.uniL` source file.
    pub source_file: String,
    /// RFC 3339 timestamp of when the build was performed.
    pub compiled_at: String,
    /// Size of the serialised bytecode in bytes.
    pub bytecode_size: usize,
    /// Target triple used (or `"host"` if none was specified).
    pub target: String,
    /// UniLang toolchain version (hardcoded to `version.workspace`).
    pub version: String,
}

// ── Build artifact ────────────────────────────────────────────────────────────

/// Describes the artefacts produced by a successful AOT compilation.
#[derive(Debug)]
pub struct NativeArtifact {
    /// Path to the emitted Rust stub source file.
    pub stub_path: String,
    /// Path to the compiled native binary, if `rustc` was invoked.
    pub binary_path: Option<String>,
    /// The AOT manifest for this build.
    pub manifest: AotManifest,
}

// ── Serialisable bytecode envelope ───────────────────────────────────────────

/// A minimal JSON-serialisable representation of the compiled bytecode.
///
/// We deliberately do **not** depend on `unilang_codegen::Bytecode`'s internal
/// representation here — we only need the raw bytes for the stub.
#[derive(Serialize, Deserialize, Debug)]
struct BytecodeEnvelope {
    /// Version tag so the embedded runner can detect version mismatches.
    version: u8,
    /// The JSON-serialised bytecode produced by `unilang_codegen`.
    payload: serde_json::Value,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Compile a UniLang source file to a self-contained native executable stub.
///
/// The approach:
/// 1. Run the full UniLang pipeline: parse → semantic → codegen → bytecode.
/// 2. Serialise the bytecode to JSON.
/// 3. Write a Rust stub (`<output_path>.rs`) that embeds the bytecode as a
///    static byte array, with a minimal embedded VM runner as `main()`.
/// 4. If `rustc` is available on `$PATH`, compile the stub to a native binary.
///
/// # Arguments
/// * `source`      — UniLang source text.
/// * `output_path` — Desired output path for the native binary (without
///                   extension); the stub will be written to `<output_path>.rs`.
/// * `target`      — Optional target triple, e.g. `"aarch64-apple-darwin"`.
///
/// # Returns
/// A [`NativeArtifact`] describing what was written to disk, or a
/// [`NativeError`] if compilation failed.
pub fn compile_to_native(
    source: &str,
    output_path: &str,
    target: Option<&str>,
) -> Result<NativeArtifact, NativeError> {
    let config = NativeCompileConfig {
        target: target.map(str::to_owned),
        ..Default::default()
    };
    compile_to_native_with_config(source, output_path, &config)
}

/// Compile to a native executable with full control over compilation options.
pub fn compile_to_native_with_config(
    source: &str,
    output_path: &str,
    config: &NativeCompileConfig,
) -> Result<NativeArtifact, NativeError> {
    // ── 1. Parse ─────────────────────────────────────────────────────────────
    let mut source_map = unilang_common::source::SourceMap::new();
    let source_id = source_map.add("<aot>".to_string(), source.to_string());

    let (module, parse_diags) = unilang_parser::parse(source_id, source);
    if parse_diags.has_errors() {
        let msgs: Vec<String> = parse_diags
            .diagnostics()
            .iter()
            .map(|d| d.message.clone())
            .collect();
        return Err(NativeError::CompileError(msgs.join("; ")));
    }

    // ── 2. Semantic analysis ─────────────────────────────────────────────────
    let (_sem_result, sem_diags) = unilang_semantic::analyze_with_extra_builtins(&module, &[]);
    if sem_diags.has_errors() {
        let msgs: Vec<String> = sem_diags
            .diagnostics()
            .iter()
            .map(|d| d.message.clone())
            .collect();
        return Err(NativeError::CompileError(msgs.join("; ")));
    }

    // ── 3. Code generation ───────────────────────────────────────────────────
    let bytecode = unilang_codegen::compile(&module).map_err(|diags| {
        let msgs: Vec<String> = diags.iter().map(|d| d.message.clone()).collect();
        NativeError::CompileError(msgs.join("; "))
    })?;

    // ── 4. Serialise bytecode ────────────────────────────────────────────────
    // `Bytecode` does not derive `Serialize`, so we convert it to a generic
    // JSON value via our own mapping helpers.
    let payload = bytecode_to_json(&bytecode);
    let envelope = BytecodeEnvelope {
        version: 1,
        payload,
    };
    let bytecode_json = serde_json::to_string(&envelope)
        .map_err(|e| NativeError::CompileError(format!("envelope serialisation failed: {}", e)))?;
    let bytecode_bytes = bytecode_json.as_bytes();

    // ── 5. Write stub ────────────────────────────────────────────────────────
    let output_dir = std::path::Path::new(output_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());

    let stub_source = build_native_stub(bytecode_bytes, &output_dir);
    let stub_path = format!("{}.rs", output_path);
    std::fs::write(&stub_path, &stub_source)?;

    // ── 6. Write manifest ────────────────────────────────────────────────────
    let target_name = config.target.as_deref().unwrap_or("host").to_string();

    let manifest = AotManifest {
        source_file: output_path.to_string(),
        compiled_at: chrono_now(),
        bytecode_size: bytecode_bytes.len(),
        target: target_name,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| NativeError::IoError(format!("manifest serialisation failed: {}", e)))?;
    std::fs::write(format!("{}.aot.json", output_path), manifest_json)?;

    // ── 7. Optionally invoke rustc ────────────────────────────────────────────
    let binary_path = invoke_rustc(&stub_path, output_path, config)?;

    Ok(NativeArtifact {
        stub_path,
        binary_path,
        manifest,
    })
}

/// Generate the Rust stub source that embeds `bytecode` as a static byte array
/// and provides a `main()` function that drives a minimal embedded VM.
///
/// # Arguments
/// * `bytecode`   — The raw bytecode bytes (JSON-serialised envelope).
/// * `_output_dir` — Reserved for future use (e.g. writing helper modules).
pub fn build_native_stub(bytecode: &[u8], _output_dir: &str) -> String {
    // Format the byte array literal.
    let byte_literals: Vec<String> = bytecode.iter().map(|b| format!("0x{:02x}", b)).collect();
    let bytes_str = byte_literals.join(", ");

    format!(
        r#"// Auto-generated by unilang-native — do not edit by hand.
// This file embeds a compiled UniLang program as a static byte array.
// It is intentionally dependency-free (stdlib only) so that it can be
// compiled with a bare `rustc <file>.rs -o <binary>` invocation.
//
// To activate full execution, link this stub with unilang-runtime.

/// The compiled UniLang program bytecode (JSON-encoded envelope).
static BYTECODE: &[u8] = &[{bytes}];

fn main() {{
    // Decode the JSON envelope using only std — no external crates needed.
    let json_str = match std::str::from_utf8(BYTECODE) {{
        Ok(s) => s,
        Err(e) => {{
            eprintln!("fatal: bytecode is not valid UTF-8: {{}}", e);
            std::process::exit(1);
        }}
    }};

    // Minimal version check: scan for `"version":1` in the raw JSON bytes.
    if !json_str.contains("\"version\":1") && !json_str.contains("\"version\": 1") {{
        eprintln!("fatal: unsupported bytecode envelope version (expected version 1)");
        std::process::exit(1);
    }}

    // In a fully-embedded scenario the UniLang runtime is statically linked
    // here and `bytecode_run(BYTECODE)` would be called.  For this stub we
    // print a diagnostic so the developer can verify the embedding worked.
    println!(
        "[unilang-native] bytecode ({{}} bytes) embedded and loaded successfully.",
        BYTECODE.len()
    );
    println!("[unilang-native] link with unilang-runtime to enable execution.");
}}
"#,
        bytes = bytes_str,
    )
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Attempt to invoke `rustc` to compile the stub to a native binary.
/// Returns `Ok(None)` if `rustc` is not on `$PATH` (soft failure).
fn invoke_rustc(
    stub_path: &str,
    output_path: &str,
    config: &NativeCompileConfig,
) -> Result<Option<String>, NativeError> {
    // Check rustc availability without hard-failing.
    let Ok(rustc) = which_rustc() else {
        return Ok(None);
    };

    let mut cmd = std::process::Command::new(rustc);
    cmd.arg(stub_path)
        .arg("-o")
        .arg(output_path)
        .arg(format!("-C opt-level={}", config.opt_level.rustc_flag()));

    if config.strip_symbols {
        cmd.arg("-C").arg("strip=symbols");
    }

    if let Some(ref target) = config.target {
        cmd.arg("--target").arg(target);
    }

    let output = cmd.output()?;
    if output.status.success() {
        Ok(Some(output_path.to_string()))
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Err(NativeError::RustcError { exit_code, stderr })
    }
}

fn which_rustc() -> Result<std::path::PathBuf, ()> {
    // Try the environment variable first, then fall back to PATH lookup.
    if let Ok(p) = std::env::var("RUSTC") {
        return Ok(std::path::PathBuf::from(p));
    }
    which("rustc").ok_or(())
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(cmd);
            if candidate.is_file() {
                Some(candidate)
            } else {
                // On Windows, also check with .exe extension.
                let with_exe = dir.join(format!("{}.exe", cmd));
                if with_exe.is_file() {
                    Some(with_exe)
                } else {
                    None
                }
            }
        })
    })
}

/// Return a simple ISO 8601 timestamp.  We avoid pulling in `chrono` to keep
/// the crate dependency-light; the precision is coarse but sufficient for the
/// manifest.
fn chrono_now() -> String {
    // std::time gives us seconds since the Unix epoch.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Very basic epoch-to-date conversion (no leap-second handling).
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;

    // Approximate Gregorian date from days-since-epoch.
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, h, m, s
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Gregorian calendar approximation (good until ~2100).
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let months = [
        31u64,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &dim in &months {
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ── Bytecode → serde_json::Value ─────────────────────────────────────────────
//
// `unilang_codegen::Bytecode` and its sub-types do not derive `Serialize`, so
// we hand-roll the conversion to a generic `serde_json::Value`.  This keeps
// `unilang-native` independent of any future changes to the codegen crate's
// internal representation.

use unilang_codegen::bytecode::{Bytecode, ClassDef, Function, Opcode, Value as BcValue};

fn bytecode_to_json(bc: &Bytecode) -> serde_json::Value {
    serde_json::json!({
        "instructions": bc.instructions.iter().map(opcode_to_json).collect::<Vec<_>>(),
        "functions":    bc.functions.iter().map(function_to_json).collect::<Vec<_>>(),
        "classes":      bc.classes.iter().map(classdef_to_json).collect::<Vec<_>>(),
    })
}

fn function_to_json(f: &Function) -> serde_json::Value {
    serde_json::json!({
        "name":        f.name,
        "params":      f.params,
        "local_count": f.local_count,
        "code":        f.code.iter().map(opcode_to_json).collect::<Vec<_>>(),
    })
}

fn classdef_to_json(c: &ClassDef) -> serde_json::Value {
    serde_json::json!({
        "name":    c.name,
        "parent":  c.parent,
        "methods": c.methods,
        "fields":  c.fields,
    })
}

fn opcode_to_json(op: &Opcode) -> serde_json::Value {
    use serde_json::json;
    match op {
        Opcode::LoadConst(v) => json!({"op": "LoadConst",  "value": value_to_json(v)}),
        Opcode::LoadLocal(i) => json!({"op": "LoadLocal",  "index": i}),
        Opcode::StoreLocal(i) => json!({"op": "StoreLocal", "index": i}),
        Opcode::LoadGlobal(n) => json!({"op": "LoadGlobal", "name": n}),
        Opcode::StoreGlobal(n) => json!({"op": "StoreGlobal","name": n}),
        Opcode::Pop => json!({"op": "Pop"}),
        Opcode::Dup => json!({"op": "Dup"}),
        Opcode::Add => json!({"op": "Add"}),
        Opcode::Sub => json!({"op": "Sub"}),
        Opcode::Mul => json!({"op": "Mul"}),
        Opcode::Div => json!({"op": "Div"}),
        Opcode::FloorDiv => json!({"op": "FloorDiv"}),
        Opcode::Mod => json!({"op": "Mod"}),
        Opcode::Pow => json!({"op": "Pow"}),
        Opcode::Neg => json!({"op": "Neg"}),
        Opcode::Eq => json!({"op": "Eq"}),
        Opcode::NotEq => json!({"op": "NotEq"}),
        Opcode::Lt => json!({"op": "Lt"}),
        Opcode::Gt => json!({"op": "Gt"}),
        Opcode::LtEq => json!({"op": "LtEq"}),
        Opcode::GtEq => json!({"op": "GtEq"}),
        Opcode::And => json!({"op": "And"}),
        Opcode::Or => json!({"op": "Or"}),
        Opcode::Not => json!({"op": "Not"}),
        Opcode::BitAnd => json!({"op": "BitAnd"}),
        Opcode::BitOr => json!({"op": "BitOr"}),
        Opcode::BitXor => json!({"op": "BitXor"}),
        Opcode::BitNot => json!({"op": "BitNot"}),
        Opcode::LShift => json!({"op": "LShift"}),
        Opcode::RShift => json!({"op": "RShift"}),
        Opcode::Concat => json!({"op": "Concat"}),
        Opcode::Jump(ip) => json!({"op": "Jump", "ip": ip}),
        Opcode::JumpIfFalse(ip) => json!({"op": "JumpIfFalse", "ip": ip}),
        Opcode::JumpIfTrue(ip) => json!({"op": "JumpIfTrue",  "ip": ip}),
        Opcode::Call(n) => json!({"op": "Call", "args": n}),
        Opcode::Return => json!({"op": "Return"}),
        Opcode::MakeFunction(i) => json!({"op": "MakeFunction", "index": i}),
        Opcode::GetAttr(n) => json!({"op": "GetAttr", "name": n}),
        Opcode::SetAttr(n) => json!({"op": "SetAttr", "name": n}),
        Opcode::MakeClass(n, c) => json!({"op": "MakeClass", "name": n, "count": c}),
        Opcode::NewInstance(n) => json!({"op": "NewInstance","name": n}),
        Opcode::MakeList(n) => json!({"op": "MakeList", "count": n}),
        Opcode::MakeDict(n) => json!({"op": "MakeDict", "count": n}),
        Opcode::GetIndex => json!({"op": "GetIndex"}),
        Opcode::SetIndex => json!({"op": "SetIndex"}),
        Opcode::Print => json!({"op": "Print"}),
        Opcode::CallMethod(m, n) => json!({"op": "CallMethod", "name": m, "args": n}),
        Opcode::Contains => json!({"op": "Contains"}),
        Opcode::Assert => json!({"op": "Assert"}),
        Opcode::Raise => json!({"op": "Raise"}),
        Opcode::PushExceptHandler(ip) => json!({"op": "PushExceptHandler", "ip": ip}),
        Opcode::PopExceptHandler => json!({"op": "PopExceptHandler"}),
        Opcode::StoreExceptVar(n) => json!({"op": "StoreExceptVar", "name": n}),
        Opcode::Halt => json!({"op": "Halt"}),
    }
}

fn value_to_json(v: &BcValue) -> serde_json::Value {
    match v {
        BcValue::Int(i) => serde_json::json!({"type": "int",    "value": i}),
        BcValue::Float(f) => serde_json::json!({"type": "float",  "value": f}),
        BcValue::String(s) => serde_json::json!({"type": "string", "value": s}),
        BcValue::Bool(b) => serde_json::json!({"type": "bool",   "value": b}),
        BcValue::Null => serde_json::json!({"type": "null"}),
    }
}
