// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! S3-compatible object storage driver.
//!
//! Implements a minimal S3 HTTP client with **AWS Signature Version 4**
//! signing.  Compatible with AWS S3, MinIO, Backblaze B2 (S3 API),
//! DigitalOcean Spaces, and any S3-compatible service.
//!
//! # UniLang functions
//! | Function | Description |
//! |---|---|
//! | `s3_connect(endpoint, bucket, access_key, secret_key, region?)` | Store connection config |
//! | `s3_put(key, content)` | Upload an object (UTF-8 string content) |
//! | `s3_get(key)` | Download an object; returns its content as String |
//! | `s3_delete(key)` | Delete an object; returns Bool true |
//! | `s3_exists(key)` | HEAD request; returns Bool |

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::{DriverCategory, UniLangDriver};

type HmacSha256 = Hmac<Sha256>;

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct S3State {
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
    region: String,
}

pub struct S3Driver {
    state: Arc<Mutex<Option<S3State>>>,
}

impl S3Driver {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for S3Driver {
    fn default() -> Self {
        Self::new()
    }
}

// ── Trait impl ───────────────────────────────────────────────────────────────

impl UniLangDriver for S3Driver {
    fn name(&self) -> &str {
        "s3"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "S3-compatible object storage (AWS S3, MinIO, Backblaze B2, DigitalOcean Spaces) with SigV4"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Other
    }
    fn exported_functions(&self) -> &'static [&'static str] {
        &["s3_connect", "s3_put", "s3_get", "s3_delete", "s3_exists"]
    }

    fn register(&self, vm: &mut VM) {
        macro_rules! arc {
            () => {
                Arc::clone(&self.state)
            };
        }

        // s3_connect(endpoint, bucket, access_key, secret_key, region?)
        {
            let state = arc!();
            vm.register_builtin("s3_connect", move |args| {
                let endpoint = str_arg(
                    args,
                    0,
                    "s3_connect(endpoint, bucket, access_key, secret_key, region?)",
                )?;
                let bucket = str_arg(
                    args,
                    1,
                    "s3_connect(endpoint, bucket, access_key, secret_key, region?)",
                )?;
                let access_key = str_arg(
                    args,
                    2,
                    "s3_connect(endpoint, bucket, access_key, secret_key, region?)",
                )?;
                let secret_key = str_arg(
                    args,
                    3,
                    "s3_connect(endpoint, bucket, access_key, secret_key, region?)",
                )?;
                let region = match args.get(4) {
                    Some(RuntimeValue::String(s)) => s.clone(),
                    _ => "us-east-1".to_string(),
                };
                *state.lock().unwrap() = Some(S3State {
                    endpoint: endpoint.trim_end_matches('/').to_string(),
                    bucket,
                    access_key,
                    secret_key,
                    region,
                });
                Ok(RuntimeValue::Bool(true))
            });
        }

        // s3_put(key, content)
        {
            let state = arc!();
            vm.register_builtin("s3_put", move |args| {
                let key = str_arg(args, 0, "s3_put(key, content)")?;
                let content = str_arg(args, 1, "s3_put(key, content)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("s3_put"))?;

                let body_bytes = content.as_bytes();
                let url = object_url(cfg, &key);
                let path = format!("/{}/{}", cfg.bucket, key.trim_start_matches('/'));

                let mut extra: BTreeMap<&str, String> = BTreeMap::new();
                extra.insert("content-type", "application/octet-stream".to_string());

                let auth = sigv4_auth(cfg, "PUT", &path, "", body_bytes, &extra)?;

                let (date_header, auth_header) = auth;
                ureq::put(&url)
                    .set("x-amz-date", &date_header.1)
                    .set("x-amz-content-sha256", &sha256_hex(body_bytes))
                    .set("Authorization", &auth_header)
                    .set("Content-Type", "application/octet-stream")
                    .send_bytes(body_bytes)
                    .map_err(|e| RuntimeError::type_error(format!("s3_put: {}", e)))?;

                Ok(RuntimeValue::Bool(true))
            });
        }

        // s3_get(key)
        {
            let state = arc!();
            vm.register_builtin("s3_get", move |args| {
                let key = str_arg(args, 0, "s3_get(key)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("s3_get"))?;

                let url = object_url(cfg, &key);
                let path = format!("/{}/{}", cfg.bucket, key.trim_start_matches('/'));

                let extra: BTreeMap<&str, String> = BTreeMap::new();
                let (date_header, auth_header) = sigv4_auth(cfg, "GET", &path, "", b"", &extra)?;

                let resp = ureq::get(&url)
                    .set("x-amz-date", &date_header.1)
                    .set("x-amz-content-sha256", EMPTY_PAYLOAD_HASH)
                    .set("Authorization", &auth_header)
                    .call()
                    .map_err(|e| RuntimeError::type_error(format!("s3_get: {}", e)))?;

                let body = resp.into_string().map_err(|e| {
                    RuntimeError::type_error(format!("s3_get: read response: {}", e))
                })?;

                Ok(RuntimeValue::String(body))
            });
        }

        // s3_delete(key)
        {
            let state = arc!();
            vm.register_builtin("s3_delete", move |args| {
                let key = str_arg(args, 0, "s3_delete(key)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("s3_delete"))?;

                let url = object_url(cfg, &key);
                let path = format!("/{}/{}", cfg.bucket, key.trim_start_matches('/'));

                let extra: BTreeMap<&str, String> = BTreeMap::new();
                let (date_header, auth_header) = sigv4_auth(cfg, "DELETE", &path, "", b"", &extra)?;

                ureq::delete(&url)
                    .set("x-amz-date", &date_header.1)
                    .set("x-amz-content-sha256", EMPTY_PAYLOAD_HASH)
                    .set("Authorization", &auth_header)
                    .call()
                    .map_err(|e| RuntimeError::type_error(format!("s3_delete: {}", e)))?;

                Ok(RuntimeValue::Bool(true))
            });
        }

        // s3_exists(key)
        {
            let state = arc!();
            vm.register_builtin("s3_exists", move |args| {
                let key = str_arg(args, 0, "s3_exists(key)")?;

                let guard = state.lock().unwrap();
                let cfg = guard.as_ref().ok_or_else(|| no_conn("s3_exists"))?;

                let url = object_url(cfg, &key);
                let path = format!("/{}/{}", cfg.bucket, key.trim_start_matches('/'));

                let extra: BTreeMap<&str, String> = BTreeMap::new();
                let (date_header, auth_header) = sigv4_auth(cfg, "HEAD", &path, "", b"", &extra)?;

                // ureq HEAD: use a raw request since ureq doesn't have a .head() shorthand
                let result = ureq::request("HEAD", &url)
                    .set("x-amz-date", &date_header.1)
                    .set("x-amz-content-sha256", EMPTY_PAYLOAD_HASH)
                    .set("Authorization", &auth_header)
                    .call();

                Ok(RuntimeValue::Bool(match result {
                    Ok(_) => true,
                    Err(ureq::Error::Status(404, _)) => false,
                    Err(e) => return Err(RuntimeError::type_error(format!("s3_exists: {}", e))),
                }))
            });
        }
    }
}

// ── AWS Signature Version 4 ───────────────────────────────────────────────────

const EMPTY_PAYLOAD_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Returns `(("x-amz-date", amz_date_value), authorization_header_value)`.
fn sigv4_auth(
    cfg: &S3State,
    method: &str,
    path: &str,
    query_string: &str,
    body: &[u8],
    extra_headers: &BTreeMap<&str, String>,
) -> Result<((&'static str, String), String), RuntimeError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (date_stamp, amz_datetime) = format_datetime(now);

    // Derive the host from the endpoint URL.
    let host = extract_host(&cfg.endpoint);

    // Build canonical header map (must be sorted).
    let mut canonical_headers: BTreeMap<String, String> = BTreeMap::new();
    canonical_headers.insert("host".to_string(), host.clone());
    canonical_headers.insert("x-amz-date".to_string(), amz_datetime.clone());

    let payload_hash = sha256_hex(body);
    canonical_headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());

    for (k, v) in extra_headers {
        canonical_headers.insert(k.to_lowercase(), v.clone());
    }

    let canonical_headers_str: String = canonical_headers
        .iter()
        .map(|(k, v)| format!("{}:{}\n", k, v.trim()))
        .collect();

    let signed_headers: String = canonical_headers
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(";");

    // Canonical request
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method,
        uri_encode_path(path),
        query_string,
        canonical_headers_str,
        signed_headers,
        payload_hash,
    );

    let cr_hash = sha256_hex(canonical_request.as_bytes());

    // Credential scope
    let scope = format!("{}/{}/s3/aws4_request", date_stamp, cfg.region);

    // String to sign
    let string_to_sign = format!("AWS4-HMAC-SHA256\n{}\n{}\n{}", amz_datetime, scope, cr_hash);

    // Signing key: HMAC chain
    let signing_key = {
        let k_date = hmac_sha256(
            format!("AWS4{}", cfg.secret_key).as_bytes(),
            date_stamp.as_bytes(),
        );
        let k_region = hmac_sha256(&k_date, cfg.region.as_bytes());
        let k_service = hmac_sha256(&k_region, b"s3");
        hmac_sha256(&k_service, b"aws4_request")
    };

    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        cfg.access_key, scope, signed_headers, signature
    );

    Ok((("x-amz-date", amz_datetime), authorization))
}

// ── Crypto helpers ────────────────────────────────────────────────────────────

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length is always valid");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

// ── Date/time helpers ─────────────────────────────────────────────────────────

/// Returns `(date_stamp "YYYYMMDD", amz_datetime "YYYYMMDDTHHMMSSZ")`.
fn format_datetime(unix_secs: u64) -> (String, String) {
    // Manual UTC decomposition without chrono dependency.
    let secs = unix_secs;
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400; // days since 1970-01-01

    let (year, month, day) = days_to_ymd(days as u32);

    let date_stamp = format!("{:04}{:02}{:02}", year, month, day);
    let amz_datetime = format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        year, month, day, h, m, s
    );
    (date_stamp, amz_datetime)
}

/// Convert days-since-epoch (1970-01-01) to (year, month, day).
fn days_to_ymd(days: u32) -> (u32, u32, u32) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m, d)
}

// ── URL helpers ───────────────────────────────────────────────────────────────

fn object_url(cfg: &S3State, key: &str) -> String {
    let key = key.trim_start_matches('/');
    format!("{}/{}/{}", cfg.endpoint, cfg.bucket, key)
}

fn extract_host(endpoint: &str) -> String {
    // Strip scheme
    let without_scheme = endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    // Strip any trailing path
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .to_string()
}

/// Percent-encode a URL path (encode everything except unreserved chars and '/').
fn uri_encode_path(path: &str) -> String {
    path.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn no_conn(func: &str) -> RuntimeError {
    RuntimeError::type_error(format!("{}: call s3_connect() first", func))
}

fn str_arg(args: &[RuntimeValue], idx: usize, sig: &str) -> Result<String, RuntimeError> {
    match args.get(idx) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::type_error(format!(
            "{}: expected string at position {}",
            sig, idx
        ))),
    }
}
