// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang pack` — bundle a project into a `.uniLpkg` ZIP archive.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;

use crate::project::load_project;

/// Entry point called from main.
pub fn cmd_pack(out: Option<&str>) {
    let (proj, _) = load_project();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Determine output path.
    let output_path: PathBuf = match out {
        Some(p) => PathBuf::from(p),
        None => cwd.join(format!("{}-{}.uniLpkg", proj.name(), proj.version())),
    };

    // Collect files.
    let files = collect_pack_files(&cwd);

    // Create ZIP archive.
    let zip_file = match File::create(&output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot create '{}': {}", output_path.display(), e);
            std::process::exit(1);
        }
    };

    let mut zip = zip::ZipWriter::new(zip_file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut count = 0usize;

    for abs_path in &files {
        let rel = match abs_path.strip_prefix(&cwd) {
            Ok(r) => r,
            Err(_) => abs_path.as_path(),
        };

        let name = rel.to_string_lossy().replace('\\', "/");

        let mut f = match File::open(abs_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("warning: cannot read '{}': {}", abs_path.display(), e);
                continue;
            }
        };

        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() {
            eprintln!("warning: cannot read contents of '{}'", abs_path.display());
            continue;
        }

        if let Err(e) = zip.start_file(name, options) {
            eprintln!("warning: zip error for '{}': {}", abs_path.display(), e);
            continue;
        }
        if let Err(e) = zip.write_all(&buf) {
            eprintln!("warning: zip write error for '{}': {}", abs_path.display(), e);
            continue;
        }
        count += 1;
    }

    if let Err(e) = zip.finish() {
        eprintln!("error: failed to finalise archive: {}", e);
        std::process::exit(1);
    }

    println!("Packed {} files → {}", count, output_path.display());
}

/// Collect all `.uniL` files (and `unilang.toml` if present) from cwd,
/// skipping `.git/` and `target/`.
fn collect_pack_files(cwd: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    // Always include unilang.toml if present.
    let toml_path = cwd.join("unilang.toml");
    if toml_path.exists() {
        files.push(toml_path);
    }

    for entry in WalkDir::new(cwd)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Skip .git/ and target/ directories.
        if entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            if name == ".git" || name == "target" {
                // We can't skip an entire directory from WalkDir without the
                // filter_entry API, but we can just skip entries whose path
                // contains these segments.
                continue;
            }
        }

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.into_path();

        // Skip if path contains .git or target segments.
        let path_str = path.to_string_lossy();
        if path_str.contains("/.git/") || path_str.contains("/target/") {
            continue;
        }

        // Include only .uniL files (unilang.toml already added above).
        if path
            .extension()
            .map(|x| x == "uniL")
            .unwrap_or(false)
        {
            files.push(path);
        }
    }

    files
}
