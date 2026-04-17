// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang build` — compile a `.uniL` file without executing it.
//! With `--incremental`, uses a SHA-256-based cache in `.unilang_cache/`.

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use unilang_common::error::Severity;
use unilang_common::source::SourceMap;

/// Entry point called from main.
pub fn cmd_build(file: &str, incremental: bool) {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", file, e);
            std::process::exit(1);
        }
    };

    if incremental {
        let hash = sha256_hex(source.as_bytes());
        let cache_dir = PathBuf::from(".unilang_cache");
        let cache_file = cache_dir.join(format!("{}.bin", hash));

        if cache_file.exists() {
            println!("(cached) {}", file);
            return;
        }

        // Compile, then write cache marker on success.
        compile_source(file, &source);

        // Create cache directory and marker file.
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            eprintln!("warning: cannot create cache dir: {}", e);
        } else if let Err(e) = std::fs::write(&cache_file, &hash) {
            eprintln!("warning: cannot write cache file: {}", e);
        }

        println!("Built: {} (cached as {})", file, cache_file.display());
    } else {
        compile_source(file, &source);
        println!("Built: {}", file);
    }
}

/// Run the full compile pipeline (parse → semantic → codegen).
/// Exits with code 1 on error.
fn compile_source(path: &str, source: &str) {
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path.to_string(), source.to_string());

    // Parse
    let (module, parse_diags) = unilang_parser::parse(source_id, source);
    if parse_diags.has_errors() {
        print_diags(path, &source_map, source_id, &parse_diags);
        std::process::exit(1);
    }

    // Semantic analysis
    let driver_funcs = unilang_drivers::default_registry().all_function_names();
    let (_result, sem_diags) =
        unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);
    if sem_diags.has_errors() {
        print_diags(path, &source_map, source_id, &sem_diags);
        std::process::exit(1);
    }

    // Codegen
    match unilang_codegen::compile(&module) {
        Ok(_) => {}
        Err(diags) => {
            for d in &diags {
                let sev = match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Hint => "hint",
                };
                eprintln!("{}: {}", sev, d.message);
            }
            std::process::exit(1);
        }
    }
}

fn print_diags(
    path: &str,
    source_map: &SourceMap,
    source_id: unilang_common::span::SourceId,
    diags: &unilang_common::error::DiagnosticBag,
) {
    let file = source_map.get(source_id);
    for d in diags.diagnostics() {
        let sev = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        };
        eprintln!("{}: {}", sev, d.message);
        for label in &d.labels {
            let lc = file.line_col(label.span.start);
            eprintln!("  --> {}:{}:{}: {}", path, lc.line, lc.col, label.message);
        }
    }
}

/// Compute SHA-256 of bytes and return a lowercase hex string.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
