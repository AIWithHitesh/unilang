// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang test` — discover and run `test_*` functions in `.uniL` files.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Entry point called from main.
pub fn cmd_test(path: Option<&str>) -> i32 {
    let files = collect_files(path);

    if files.is_empty() {
        eprintln!("No .uniL files found.");
        return 0;
    }

    let mut passed = 0usize;
    let mut failed = 0usize;

    for file in &files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read '{}': {}", file.display(), e);
                continue;
            }
        };

        let test_fns = find_test_functions(&source);
        if test_fns.is_empty() {
            continue;
        }

        for test_name in &test_fns {
            match run_test(file, &source, test_name) {
                true => {
                    println!("PASS {}", test_name);
                    passed += 1;
                }
                false => {
                    println!("FAIL {}", test_name);
                    failed += 1;
                }
            }
        }
    }

    let total = passed + failed;
    println!();
    println!("{} passed, {} failed out of {} tests", passed, failed, total);

    if failed > 0 { 1 } else { 0 }
}

/// Collect `.uniL` files from the given path (file, dir, or cwd).
fn collect_files(path: Option<&str>) -> Vec<PathBuf> {
    let target = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    if target.is_file() {
        return vec![target];
    }

    WalkDir::new(&target)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map(|x| x == "uniL")
                    .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

/// Find all `test_*` function names in source text.
fn find_test_functions(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("def ") {
            let func_name = rest
                .split(|c: char| c == '(' || c == ':' || c == ' ')
                .next()
                .unwrap_or("")
                .trim();
            if func_name.starts_with("test_") {
                names.push(func_name.to_string());
            }
        }
    }
    names
}

/// Build a wrapper snippet that calls `test_name()` and run it through the VM.
/// Returns `true` on success (no RuntimeError), `false` on any error/panic.
fn run_test(file: &Path, source: &str, test_name: &str) -> bool {
    // Append a call to the test function.
    let snippet = format!("{}\n{test_name}()\n", source, test_name = test_name);

    let result = std::panic::catch_unwind(|| execute_snippet(&snippet));

    match result {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => {
            eprintln!("  panic in {} ({})", test_name, file.display());
            false
        }
    }
}

/// Full pipeline: parse → semantic → codegen → VM.  Returns true on clean exit.
fn execute_snippet(source: &str) -> bool {
    use unilang_common::source::SourceMap;
    use unilang_runtime::error::ErrorKind;

    let mut source_map = SourceMap::new();
    let source_id = source_map.add("<test>".to_string(), source.to_string());

    let (module, parse_diags) = unilang_parser::parse(source_id, source);
    if parse_diags.has_errors() {
        return false;
    }

    let driver_funcs = unilang_drivers::default_registry().all_function_names();
    let (_result, sem_diags) =
        unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);
    if sem_diags.has_errors() {
        return false;
    }

    let bytecode = match unilang_codegen::compile(&module) {
        Ok(bc) => bc,
        Err(_) => return false,
    };

    let mut vm = unilang_runtime::vm::VM::new();
    unilang_stdlib::register_builtins(&mut vm);
    let drivers = unilang_drivers::default_registry();
    drivers.register_all(&mut vm);

    match vm.run(&bytecode) {
        Ok(_) => true,
        Err(e) => {
            if e.kind == ErrorKind::Halt {
                true
            } else {
                eprintln!("  runtime error: {}", e.message);
                false
            }
        }
    }
}
