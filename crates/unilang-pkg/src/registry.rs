// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Registry client for the `unilang.dev` package registry.
//!
//! All functions in this module communicate with the live registry over HTTPS.
//! They gracefully return [`PkgError::NetworkError`] when offline so that
//! callers can fall back to cached data.

use crate::{PkgError, RegistryPackage};

const REGISTRY_BASE: &str = "https://registry.unilang.dev";

/// Fetch metadata for a single package by name.
///
/// Returns a [`RegistryPackage`] with all published versions and download
/// statistics.
pub fn package_info(name: &str) -> Result<RegistryPackage, PkgError> {
    let url = format!("{}/packages/{}", REGISTRY_BASE, name);
    let body = get_json_body(&url)?;
    serde_json::from_str(&body).map_err(|e| PkgError::SerdeError(e.to_string()))
}

/// Return the latest published version string for `name`.
///
/// This is a convenience wrapper around [`package_info`].
pub fn latest_version(name: &str) -> Result<String, PkgError> {
    let info = package_info(name)?;
    if info.latest.is_empty() {
        Err(PkgError::NotFound(format!(
            "no published versions for package '{}'",
            name
        )))
    } else {
        Ok(info.latest)
    }
}

/// Return all known versions of a package (newest first).
pub fn all_versions(name: &str) -> Result<Vec<String>, PkgError> {
    let info = package_info(name)?;
    Ok(info.versions)
}

/// Fetch the expected SHA-256 checksum for a specific package release.
///
/// The registry serves checksums at `GET /packages/{name}/{version}.sha256`.
/// Returns an empty string if the registry does not provide a checksum
/// (the caller should skip verification in that case).
pub fn fetch_checksum(name: &str, version: &str) -> Result<String, PkgError> {
    let url = format!("{}/packages/{}/{}.sha256", REGISTRY_BASE, name, version);
    match get_json_body(&url) {
        Ok(body) => Ok(body.trim().to_string()),
        // 404 means the registry doesn't provide a checksum for this release.
        Err(PkgError::HttpError { status: 404, .. }) => Ok(String::new()),
        // Any other error is propagated.
        Err(e) => Err(e),
    }
}

/// Download the raw bytes of a package archive.
///
/// Equivalent to `GET /packages/{name}/{version}.zip`.
pub fn download_package(name: &str, version: &str) -> Result<Vec<u8>, PkgError> {
    let url = format!("{}/packages/{}/{}.zip", REGISTRY_BASE, name, version);
    let response = ureq::get(&url).call().map_err(crate::map_ureq_error)?;

    let mut bytes = Vec::new();
    use std::io::Read;
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| PkgError::IoError(e.to_string()))?;
    Ok(bytes)
}

/// Authenticate with the registry and return a session token.
///
/// POSTs `{ "username": ..., "password": ... }` to `/auth/login`.
pub fn login(username: &str, password: &str) -> Result<String, PkgError> {
    let url = format!("{}/auth/login", REGISTRY_BASE);
    let body = serde_json::json!({
        "username": username,
        "password": password,
    });

    let response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(&body).unwrap())
        .map_err(crate::map_ureq_error)?;

    let text = response
        .into_string()
        .map_err(|e| PkgError::IoError(e.to_string()))?;

    let val: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| PkgError::SerdeError(e.to_string()))?;

    val["token"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| PkgError::SerdeError("missing `token` in login response".to_string()))
}

/// Verify that `token` is still valid with the registry.
///
/// Returns `Ok(true)` if valid, `Ok(false)` if the token has expired.
pub fn verify_token(token: &str) -> Result<bool, PkgError> {
    let url = format!("{}/auth/verify", REGISTRY_BASE);
    match ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(crate::map_ureq_error)
    {
        Ok(_) => Ok(true),
        Err(PkgError::HttpError { status: 401, .. }) => Ok(false),
        Err(e) => Err(e),
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Issue a GET request and return the response body as a `String`.
///
/// In ureq 2.x, non-2xx status codes surface as `Err(ureq::Error::Status(code, _))`,
/// which `map_ureq_error` converts into `PkgError::HttpError { status, body }`.
/// A 404 therefore arrives as `Err(PkgError::HttpError { status: 404, .. })`.
pub(crate) fn get_json_body(url: &str) -> Result<String, PkgError> {
    let response = ureq::get(url).call().map_err(crate::map_ureq_error)?;

    response
        .into_string()
        .map_err(|e| PkgError::IoError(e.to_string()))
}
