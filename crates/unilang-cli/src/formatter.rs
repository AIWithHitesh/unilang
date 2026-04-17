// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang fmt` — format `.uniL` source files.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Entry point called from main.
pub fn cmd_fmt(path: Option<&str>, write: bool) {
    let target = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    if target.is_file() {
        format_file(&target, write);
    } else {
        let files = collect_unilang_files(&target);
        for f in &files {
            format_file(f, write);
        }
    }
}

/// Collect all `.uniL` files under `dir`.
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

/// Format a single file, either printing to stdout or writing back.
fn format_file(path: &Path, write: bool) {
    let original = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", path.display(), e);
            return;
        }
    };

    let formatted = format_source(&original);

    if write {
        if formatted != original {
            if let Err(e) = std::fs::write(path, &formatted) {
                eprintln!("error: cannot write '{}': {}", path.display(), e);
            } else {
                println!("Formatted: {}", path.display());
            }
        }
    } else {
        print!("{}", formatted);
    }
}

/// Apply formatting rules to source text:
/// 1. Replace each leading tab with 4 spaces.
/// 2. Trim trailing whitespace from each line.
/// 3. Collapse >2 consecutive blank lines to 2.
/// 4. Ensure the file ends with exactly one newline.
pub fn format_source(source: &str) -> String {
    // Step 1 & 2: normalise each line.
    let lines: Vec<String> = source
        .lines()
        .map(|line| {
            // Replace leading tabs with 4 spaces each.
            let mut result = String::new();
            let mut chars = line.chars().peekable();
            // Leading whitespace: expand tabs.
            while let Some(&c) = chars.peek() {
                if c == '\t' {
                    result.push_str("    ");
                    chars.next();
                } else if c == ' ' {
                    result.push(' ');
                    chars.next();
                } else {
                    break;
                }
            }
            // Rest of the line.
            for c in chars {
                result.push(c);
            }
            // Trim trailing whitespace.
            result.trim_end().to_string()
        })
        .collect();

    // Step 3: collapse >2 consecutive blank lines.
    let mut output_lines: Vec<&str> = Vec::with_capacity(lines.len());
    let mut blank_run = 0usize;
    for line in &lines {
        if line.is_empty() {
            blank_run += 1;
            if blank_run <= 2 {
                output_lines.push(line.as_str());
            }
        } else {
            blank_run = 0;
            output_lines.push(line.as_str());
        }
    }

    // Step 4: join and ensure exactly one trailing newline.
    let mut result = output_lines.join("\n");
    // Strip any trailing newlines, then add exactly one.
    while result.ends_with('\n') {
        result.pop();
    }
    result.push('\n');
    result
}
