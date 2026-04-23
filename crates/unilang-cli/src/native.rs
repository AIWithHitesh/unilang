// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang build --native` — AOT compile a .uniL file to a native binary.

use std::process;
use unilang_native::{NativeCompileConfig, OptLevel};

/// Entry point for `unilang build-native <file> [options]`.
pub fn cmd_build_native(
    file: &str,
    out: Option<&str>,
    target: Option<&str>,
    opt: &str,
    strip: bool,
) {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", file, e);
            process::exit(1);
        }
    };

    // Derive default output path from the source file name.
    let default_out;
    let output_path = match out {
        Some(p) => p,
        None => {
            default_out = std::path::Path::new(file)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "output".to_string());
            &default_out
        }
    };

    let opt_level = match opt {
        "0" => OptLevel::None,
        "1" => OptLevel::Light,
        "2" => OptLevel::Default,
        "3" => OptLevel::Aggressive,
        "s" | "size" => OptLevel::Size,
        other => {
            eprintln!("warning: unknown opt level '{}', using 2", other);
            OptLevel::Default
        }
    };

    let config = NativeCompileConfig {
        target: target.map(str::to_owned),
        opt_level,
        strip_symbols: strip,
    };

    eprintln!("compiling {} → native binary …", file);

    match unilang_native::compile_to_native_with_config(&source, output_path, &config) {
        Ok(artifact) => {
            eprintln!("stub written: {}", artifact.stub_path);
            if let Some(ref bin) = artifact.binary_path {
                eprintln!("native binary: {}", bin);
            } else {
                eprintln!(
                    "note: rustc not found on PATH — stub ready at {}",
                    artifact.stub_path
                );
                eprintln!(
                    "      compile manually:  rustc {} -o {}",
                    artifact.stub_path, output_path
                );
            }
            eprintln!(
                "manifest: {}.aot.json ({} bytes of bytecode)",
                output_path, artifact.manifest.bytecode_size
            );
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
