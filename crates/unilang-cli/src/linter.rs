// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang lint` — semantic analysis + style checks for `.uniL` files.

use std::path::{Path, PathBuf};
use unilang_common::error::Severity;
use unilang_common::source::SourceMap;
use walkdir::WalkDir;

/// Entry point called from main.  Returns exit code: 0 = ok, 1 = errors, 2 = warnings only.
pub fn cmd_lint(path: Option<&str>) -> i32 {
    let target = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let files = if target.is_file() {
        vec![target]
    } else {
        collect_unilang_files(&target)
    };

    if files.is_empty() {
        eprintln!("No .uniL files found.");
        return 0;
    }

    let mut any_errors = false;
    let mut any_warnings = false;

    for file in &files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read '{}': {}", file.display(), e);
                any_errors = true;
                continue;
            }
        };

        let (file_errors, file_warnings) = lint_file(file, &source);
        if file_errors {
            any_errors = true;
        }
        if file_warnings {
            any_warnings = true;
        }
    }

    if any_errors {
        1
    } else if any_warnings {
        2
    } else {
        0
    }
}

/// Collect all `.uniL` files recursively under `dir`.
fn collect_unilang_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
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

/// Lint a single file.  Returns `(has_errors, has_warnings)`.
fn lint_file(path: &Path, source: &str) -> (bool, bool) {
    let path_str = path.display().to_string();
    let mut has_errors = false;
    let mut has_warnings = false;

    // --- Semantic analysis ---
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path_str.clone(), source.to_string());

    let (module, parse_diags) = unilang_parser::parse(source_id, source);

    for d in parse_diags.diagnostics() {
        let sev = severity_str(d.severity);
        let file = source_map.get(source_id);
        if let Some(label) = d.labels.first() {
            let lc = file.line_col(label.span.start);
            println!("{}:{}:{}: {}: {}", path_str, lc.line, lc.col, sev, d.message);
        } else {
            println!("{}: {}: {}", path_str, sev, d.message);
        }
        if d.severity == Severity::Error {
            has_errors = true;
        } else {
            has_warnings = true;
        }
    }

    if !parse_diags.has_errors() {
        let driver_funcs = unilang_drivers::default_registry().all_function_names();
        let (_result, sem_diags) =
            unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);

        for d in sem_diags.diagnostics() {
            let sev = severity_str(d.severity);
            let file = source_map.get(source_id);
            if let Some(label) = d.labels.first() {
                let lc = file.line_col(label.span.start);
                println!("{}:{}:{}: {}: {}", path_str, lc.line, lc.col, sev, d.message);
            } else {
                println!("{}: {}: {}", path_str, sev, d.message);
            }
            if d.severity == Severity::Error {
                has_errors = true;
            } else {
                has_warnings = true;
            }
        }
    }

    // --- Style checks ---
    for (lineno, line) in source.lines().enumerate() {
        let n = lineno + 1; // 1-based

        // Line length > 120
        if line.len() > 120 {
            println!(
                "{}:{}: warning [line-length]: line {} exceeds 120 characters",
                path_str, n, n
            );
            has_warnings = true;
        }

        // Trailing whitespace
        if line != line.trim_end() {
            println!(
                "{}:{}: warning [trailing-whitespace]: line {} has trailing whitespace",
                path_str, n, n
            );
            has_warnings = true;
        }

        // TODO / FIXME comments
        let upper = line.to_uppercase();
        if upper.contains("# TODO") || upper.contains("#TODO")
            || upper.contains("# FIXME") || upper.contains("#FIXME")
        {
            println!(
                "{}:{}: hint [todo-comment]: TODO found at line {}",
                path_str, n, n
            );
        }
    }

    (has_errors, has_warnings)
}

fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Hint => "hint",
    }
}
