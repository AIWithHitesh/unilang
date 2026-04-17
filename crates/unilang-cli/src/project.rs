// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Project configuration (`unilang.toml`) types and helpers.
//! Also handles `unilang config show|init`.
//!
//! We parse the TOML manually (no external toml crate) to avoid pulling
//! in transitive dependencies that require edition2024 on Cargo < 1.85.

use std::collections::HashMap;
use std::path::PathBuf;

// ── Config structs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct PackageMeta {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Project {
    pub package: PackageMeta,
    pub dependencies: HashMap<String, String>,
    pub features: HashMap<String, Vec<String>>,
}

impl Project {
    pub fn name(&self) -> &str {
        self.package.name.as_deref().unwrap_or("project")
    }

    pub fn version(&self) -> &str {
        self.package.version.as_deref().unwrap_or("0.1.0")
    }
}

// ── Minimal TOML parser ────────────────────────────────────────────────────────

/// Parse a `unilang.toml`-shaped file into a `Project`.
/// Supports only simple `key = "value"` entries under `[package]`,
/// `[dependencies]`, and `[features]`.
pub fn parse_toml(content: &str) -> Project {
    let mut proj = Project::default();
    let mut current_section = "";

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and blank lines.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header.
        if let Some(sec) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current_section = sec.trim();
            continue;
        }

        // Key = value pairs.
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let raw_val = line[eq_pos + 1..].trim();

            match current_section {
                "package" => {
                    let val = unquote(raw_val).unwrap_or_else(|| raw_val.to_string());
                    match key {
                        "name" => proj.package.name = Some(val),
                        "version" => proj.package.version = Some(val),
                        "description" => proj.package.description = Some(val),
                        "authors" => {
                            proj.package.authors = parse_string_array(raw_val);
                        }
                        _ => {}
                    }
                }
                "dependencies" => {
                    let val = unquote(raw_val).unwrap_or_else(|| raw_val.to_string());
                    proj.dependencies.insert(key.to_string(), val);
                }
                "features" => {
                    proj.features
                        .insert(key.to_string(), parse_string_array(raw_val));
                }
                _ => {}
            }
        }
    }

    proj
}

/// Strip surrounding `"` or `'` quotes from a string token.
fn unquote(s: &str) -> Option<String> {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        Some(s[1..s.len() - 1].to_string())
    } else {
        None
    }
}

/// Parse a TOML inline array of strings: `["a", "b"]` → `vec!["a", "b"]`.
fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    let inner = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        return Vec::new();
    };

    inner
        .split(',')
        .filter_map(|item| unquote(item.trim()))
        .collect()
}

// ── Load helper ────────────────────────────────────────────────────────────────

/// Try to load `unilang.toml` from the current directory.
/// Returns a default `Project` if the file does not exist.
pub fn load_project() -> (Project, Option<PathBuf>) {
    let path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("unilang.toml");

    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let proj = parse_toml(&content);
        (proj, Some(path))
    } else {
        (Project::default(), None)
    }
}

// ── CLI commands ───────────────────────────────────────────────────────────────

/// `unilang config show` — pretty-print the parsed `unilang.toml`.
pub fn cmd_config_show() {
    let (proj, maybe_path) = load_project();
    match maybe_path {
        Some(p) => println!("# {}\n", p.display()),
        None => println!("# (no unilang.toml found — using defaults)\n"),
    }

    println!("[package]");
    println!("  name        = \"{}\"", proj.name());
    println!("  version     = \"{}\"", proj.version());
    if let Some(desc) = &proj.package.description {
        println!("  description = \"{}\"", desc);
    }
    if !proj.package.authors.is_empty() {
        println!("  authors     = {:?}", proj.package.authors);
    }

    if !proj.dependencies.is_empty() {
        println!("\n[dependencies]");
        let mut deps: Vec<_> = proj.dependencies.iter().collect();
        deps.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in deps {
            println!("  {} = \"{}\"", k, v);
        }
    }

    if !proj.features.is_empty() {
        println!("\n[features]");
        let mut feats: Vec<_> = proj.features.iter().collect();
        feats.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in feats {
            println!("  {} = {:?}", k, v);
        }
    }
}

/// `unilang config init` — scaffold a new `unilang.toml` in current dir.
pub fn cmd_config_init() {
    let path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("unilang.toml");

    if path.exists() {
        eprintln!("unilang.toml already exists at {}", path.display());
        std::process::exit(1);
    }

    let content = r#"[package]
name = "my-project"
version = "0.1.0"
description = "My UniLang project"
authors = ["Author Name"]

[dependencies]
sqlite = "1.0"
redis = "1.0"

[features]
web = []
"#;

    match std::fs::write(&path, content) {
        Ok(_) => println!("Created {}", path.display()),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}
