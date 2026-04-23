// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang package manager — publish, install, search, and manage packages
//! from the `unilang.dev` registry.
//!
//! # Quick-start
//! ```ignore
//! // Install a package
//! unilang_pkg::install("http-client", None, ".")?;
//!
//! // Search the registry
//! let results = unilang_pkg::search("http")?;
//!
//! // Publish your own package
//! let manifest = unilang_pkg::read_manifest(".")?;
//! unilang_pkg::publish(&manifest, ".", "my-auth-token")?;
//! ```

pub mod registry;
pub mod resolver;

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;

// ── Registry base URL ─────────────────────────────────────────────────────────

const REGISTRY_BASE: &str = "https://registry.unilang.dev";

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors produced by the UniLang package manager.
#[derive(Debug)]
pub enum PkgError {
    /// A network request failed (connection refused, timeout, …).
    NetworkError(String),
    /// The server returned a non-2xx status code.
    HttpError { status: u16, body: String },
    /// An I/O operation failed.
    IoError(String),
    /// JSON (de)serialisation failed.
    SerdeError(String),
    /// SHA-256 checksum mismatch.
    ChecksumMismatch { expected: String, got: String },
    /// A package manifest is missing required fields or contains invalid data.
    InvalidManifest(String),
    /// The requested package or version was not found in the registry.
    NotFound(String),
    /// Dependency resolution failed (cycle detected or version conflict).
    ResolutionError(String),
}

impl fmt::Display for PkgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PkgError::NetworkError(msg) => write!(f, "network error: {}", msg),
            PkgError::HttpError { status, body } => {
                write!(f, "HTTP {} error: {}", status, body)
            }
            PkgError::IoError(msg) => write!(f, "I/O error: {}", msg),
            PkgError::SerdeError(msg) => write!(f, "serialisation error: {}", msg),
            PkgError::ChecksumMismatch { expected, got } => {
                write!(f, "checksum mismatch — expected {}, got {}", expected, got)
            }
            PkgError::InvalidManifest(msg) => write!(f, "invalid manifest: {}", msg),
            PkgError::NotFound(name) => write!(f, "package not found: {}", name),
            PkgError::ResolutionError(msg) => write!(f, "dependency resolution error: {}", msg),
        }
    }
}

impl std::error::Error for PkgError {}

impl From<std::io::Error> for PkgError {
    fn from(e: std::io::Error) -> Self {
        PkgError::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for PkgError {
    fn from(e: serde_json::Error) -> Self {
        PkgError::SerdeError(e.to_string())
    }
}

// ── Data structures ───────────────────────────────────────────────────────────

/// The `unilang.toml` manifest for a UniLang package.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackageManifest {
    /// Package name (lowercase, hyphen-separated, e.g. `"http-client"`).
    pub name: String,
    /// Semantic version string, e.g. `"1.2.3"`.
    pub version: String,
    /// Short human-readable description.
    pub description: String,
    /// Author name or email.
    pub author: String,
    /// SPDX licence identifier, e.g. `"Apache-2.0"`.
    pub license: String,
    /// Direct dependencies: `{ "dep-name" => "version-req" }`.
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// Path to the main `.uniL` entry file (relative to package root).
    pub entry: Option<String>,
}

impl PackageManifest {
    /// Validate that all required fields are non-empty.
    pub fn validate(&self) -> Result<(), PkgError> {
        if self.name.is_empty() {
            return Err(PkgError::InvalidManifest(
                "field `name` is required".to_string(),
            ));
        }
        if self.version.is_empty() {
            return Err(PkgError::InvalidManifest(
                "field `version` is required".to_string(),
            ));
        }
        if self.author.is_empty() {
            return Err(PkgError::InvalidManifest(
                "field `author` is required".to_string(),
            ));
        }
        Ok(())
    }
}

/// A package record returned by the registry's search / info endpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryPackage {
    /// Package name.
    pub name: String,
    /// Latest published version.
    pub latest: String,
    /// All published versions, newest first.
    pub versions: Vec<String>,
    /// Short description.
    pub description: String,
    /// Total download count.
    pub downloads: u64,
}

/// A lock-file entry representing a resolved, downloaded package.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub url: String,
}

/// The full `unilang.lock` file.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LockFile {
    #[serde(default)]
    pub packages: Vec<LockEntry>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Download and install a package from the registry.
///
/// # Arguments
/// * `package`    — Package name (optionally `"name@version"`).
/// * `version`    — Explicit version, overrides `@version` in `package`.
///                  Pass `None` to use the latest published version.
/// * `target_dir` — Project root directory.  The package is installed into
///                  `<target_dir>/unilang_packages/<name>/`.
pub fn install(package: &str, version: Option<&str>, target_dir: &str) -> Result<(), PkgError> {
    // Parse `name[@version]`.
    let (name, inline_version) = parse_package_spec(package);
    let version_str = version.or(inline_version.as_deref()).map(str::to_owned);

    // Resolve the version to use.
    let resolved_version = match version_str {
        Some(v) => v,
        None => registry::latest_version(&name)?,
    };

    // Download the zip archive.
    let url = format!(
        "{}/packages/{}/{}.zip",
        REGISTRY_BASE, name, resolved_version
    );
    let (zip_bytes, checksum) = download_with_checksum(&url)?;

    // Verify checksum (the registry returns the expected SHA-256 in a header
    // or a companion `.sha256` URL; here we verify what we downloaded).
    let expected_checksum = registry::fetch_checksum(&name, &resolved_version)?;
    if !expected_checksum.is_empty() && checksum != expected_checksum {
        return Err(PkgError::ChecksumMismatch {
            expected: expected_checksum,
            got: checksum.clone(),
        });
    }

    // Extract into `<target_dir>/unilang_packages/<name>/`.
    let install_dir = Path::new(target_dir).join("unilang_packages").join(&name);
    std::fs::create_dir_all(&install_dir)?;
    extract_zip(&zip_bytes, &install_dir)?;

    // Update / create `unilang.lock`.
    let lock_path = Path::new(target_dir).join("unilang.lock");
    let mut lock = read_lock(&lock_path);
    lock.packages.retain(|e| e.name != name);
    lock.packages.push(LockEntry {
        name: name.clone(),
        version: resolved_version,
        checksum,
        url,
    });
    write_lock(&lock_path, &lock)?;

    println!("installed: {} into {}", name, install_dir.display());
    Ok(())
}

/// Publish a package to the registry.
///
/// # Arguments
/// * `manifest`     — The parsed `unilang.toml` for the package.
/// * `package_dir`  — Path to the package root directory.
/// * `token`        — Authentication token for the registry.
pub fn publish(
    manifest: &PackageManifest,
    package_dir: &str,
    token: &str,
) -> Result<String, PkgError> {
    manifest.validate()?;

    // Create an in-memory zip of the package directory.
    let zip_bytes = create_zip(package_dir)?;

    // Compute SHA-256 checksum.
    let checksum = sha256_hex(&zip_bytes);

    // POST to the registry.
    let url = format!("{}/publish", REGISTRY_BASE);
    let body = serde_json::json!({
        "name":        manifest.name,
        "version":     manifest.version,
        "description": manifest.description,
        "author":      manifest.author,
        "license":     manifest.license,
        "checksum":    checksum,
        "archive":     base64_encode(&zip_bytes),
    });

    let _response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(&body)?)
        .map_err(|e| map_ureq_error(e))?;

    let published_url = format!(
        "https://unilang.dev/packages/{}/{}",
        manifest.name, manifest.version
    );
    println!(
        "published: {}@{} — {}",
        manifest.name, manifest.version, published_url
    );
    Ok(published_url)
}

/// Search the registry for packages matching `query`.
///
/// Returns up to 20 results sorted by download count (highest first).
pub fn search(query: &str) -> Result<Vec<RegistryPackage>, PkgError> {
    let url = format!("{}/search?q={}", REGISTRY_BASE, url_encode(query));
    let body = ureq::get(&url)
        .call()
        .map_err(|e| map_ureq_error(e))?
        .into_string()
        .map_err(|e| PkgError::IoError(e.to_string()))?;

    let packages: Vec<RegistryPackage> =
        serde_json::from_str(&body).map_err(|e| PkgError::SerdeError(e.to_string()))?;
    Ok(packages)
}

/// List all packages installed in `project_dir/unilang_packages/`.
pub fn list_installed(project_dir: &str) -> Result<Vec<PackageManifest>, PkgError> {
    let packages_dir = Path::new(project_dir).join("unilang_packages");
    if !packages_dir.exists() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();
    for entry in std::fs::read_dir(&packages_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("unilang.toml");
        if !manifest_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&manifest_path)?;
        match toml_parse_manifest(&content) {
            Ok(m) => manifests.push(m),
            Err(e) => eprintln!("warning: skipping {}: {}", manifest_path.display(), e),
        }
    }
    Ok(manifests)
}

/// Scaffold a `unilang.toml` in `dir` with a template `PackageManifest`.
///
/// Errors if `unilang.toml` already exists.
pub fn init_manifest(name: &str, dir: &str) -> Result<(), PkgError> {
    let manifest_path = Path::new(dir).join("unilang.toml");
    if manifest_path.exists() {
        return Err(PkgError::InvalidManifest(format!(
            "unilang.toml already exists at {}",
            manifest_path.display()
        )));
    }

    let template = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
description = "A new UniLang package"
author = "Your Name <you@example.com>"
license = "Apache-2.0"
entry = "main.uniL"

[dependencies]
# example = "1.0"
"#,
        name = name,
    );

    std::fs::create_dir_all(dir)?;
    std::fs::write(&manifest_path, template)?;
    println!("created: {}", manifest_path.display());
    Ok(())
}

/// Read and parse a `unilang.toml` file from `project_dir`.
pub fn read_manifest(project_dir: &str) -> Result<PackageManifest, PkgError> {
    let path = Path::new(project_dir).join("unilang.toml");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| PkgError::IoError(format!("cannot read {}: {}", path.display(), e)))?;
    toml_parse_manifest(&content)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Parse `"name"` or `"name@version"` into `(name, Option<version>)`.
fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(at) = spec.find('@') {
        let name = spec[..at].to_string();
        let ver = spec[at + 1..].to_string();
        (name, Some(ver))
    } else {
        (spec.to_string(), None)
    }
}

/// Download `url`, return `(bytes, sha256_hex)`.
fn download_with_checksum(url: &str) -> Result<(Vec<u8>, String), PkgError> {
    let response = ureq::get(url).call().map_err(|e| map_ureq_error(e))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| PkgError::IoError(e.to_string()))?;

    let checksum = sha256_hex(&bytes);
    Ok((bytes, checksum))
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Very minimal base64 encoder (avoids an extra dependency).
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 6) & 0x3f) as usize] as char);
        out.push(CHARS[(n & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        out.push_str("==");
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 6) & 0x3f) as usize] as char);
        out.push('=');
    }
    out
}

/// URL-encode a query string (spaces → `%20`, etc.).
fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            b => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Extract a zip archive (`zip_bytes`) into `dest_dir`.
fn extract_zip(zip_bytes: &[u8], dest_dir: &Path) -> Result<(), PkgError> {
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| PkgError::IoError(format!("cannot open zip: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| PkgError::IoError(e.to_string()))?;

        let out_path = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| PkgError::IoError(e.to_string()))?;
            out_file.write_all(&buf)?;
        }
    }
    Ok(())
}

/// Create an in-memory zip of `src_dir`.
fn create_zip(src_dir: &str) -> Result<Vec<u8>, PkgError> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    add_dir_to_zip(&mut zip, Path::new(src_dir), Path::new(src_dir))?;
    let cursor = zip.finish().map_err(|e| PkgError::IoError(e.to_string()))?;
    Ok(cursor.into_inner())
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
    base: &Path,
    current: &Path,
) -> Result<(), PkgError> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(base).unwrap_or(&path);
        let name = rel.to_string_lossy();

        if path.is_dir() {
            let options = zip::write::SimpleFileOptions::default();
            zip.add_directory(format!("{}/", name), options)
                .map_err(|e| PkgError::IoError(e.to_string()))?;
            add_dir_to_zip(zip, base, &path)?;
        } else {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            zip.start_file(name.as_ref(), options)
                .map_err(|e| PkgError::IoError(e.to_string()))?;
            let data = std::fs::read(&path)?;
            zip.write_all(&data)?;
        }
    }
    Ok(())
}

fn read_lock(path: &Path) -> LockFile {
    if !path.exists() {
        return LockFile::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_lock(path: &Path, lock: &LockFile) -> Result<(), PkgError> {
    let json = serde_json::to_string_pretty(lock)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Minimal TOML parser for `unilang.toml` — avoids a heavy `toml` dependency.
/// Supports the `[package]` and `[dependencies]` sections.
fn toml_parse_manifest(content: &str) -> Result<PackageManifest, PkgError> {
    let mut name = String::new();
    let mut version = String::new();
    let mut description = String::new();
    let mut author = String::new();
    let mut license = String::new();
    let mut entry: Option<String> = None;
    let mut dependencies: HashMap<String, String> = HashMap::new();

    let mut section = "";
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']');
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"').to_string();
            match section {
                "package" => match key {
                    "name" => name = val,
                    "version" => version = val,
                    "description" => description = val,
                    "author" => author = val,
                    "license" => license = val,
                    "entry" => entry = Some(val),
                    _ => {}
                },
                "dependencies" => {
                    dependencies.insert(key.to_string(), val);
                }
                _ => {}
            }
        }
    }

    if name.is_empty() {
        return Err(PkgError::InvalidManifest(
            "missing `name` in [package]".to_string(),
        ));
    }

    Ok(PackageManifest {
        name,
        version,
        description,
        author,
        license,
        dependencies,
        entry,
    })
}

/// Map a `ureq` error to a `PkgError`.
pub(crate) fn map_ureq_error(e: ureq::Error) -> PkgError {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            PkgError::HttpError { status: code, body }
        }
        ureq::Error::Transport(t) => PkgError::NetworkError(t.to_string()),
    }
}
