// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! DateTime built-in functions: datetime_now, datetime_utcnow, datetime_parse,
//! datetime_format, datetime_add, datetime_diff_seconds, timestamp_to_datetime,
//! datetime_to_timestamp.

use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};
use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register datetime built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("datetime_now", builtin_datetime_now);
    vm.register_builtin("datetime_utcnow", builtin_datetime_utcnow);
    vm.register_builtin("datetime_parse", builtin_datetime_parse);
    vm.register_builtin("datetime_format", builtin_datetime_format);
    vm.register_builtin("datetime_add", builtin_datetime_add);
    vm.register_builtin("datetime_diff_seconds", builtin_datetime_diff_seconds);
    vm.register_builtin("timestamp_to_datetime", builtin_timestamp_to_datetime);
    vm.register_builtin("datetime_to_timestamp", builtin_datetime_to_timestamp);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a NaiveDateTime into a RuntimeValue::Dict with standard keys.
fn naive_datetime_to_dict(dt: NaiveDateTime, timestamp: f64) -> RuntimeValue {
    RuntimeValue::Dict(vec![
        (
            RuntimeValue::String("year".to_string()),
            RuntimeValue::Int(dt.year() as i64),
        ),
        (
            RuntimeValue::String("month".to_string()),
            RuntimeValue::Int(dt.month() as i64),
        ),
        (
            RuntimeValue::String("day".to_string()),
            RuntimeValue::Int(dt.day() as i64),
        ),
        (
            RuntimeValue::String("hour".to_string()),
            RuntimeValue::Int(dt.hour() as i64),
        ),
        (
            RuntimeValue::String("minute".to_string()),
            RuntimeValue::Int(dt.minute() as i64),
        ),
        (
            RuntimeValue::String("second".to_string()),
            RuntimeValue::Int(dt.second() as i64),
        ),
        (
            RuntimeValue::String("microsecond".to_string()),
            RuntimeValue::Int((dt.nanosecond() / 1_000) as i64),
        ),
        (
            RuntimeValue::String("timestamp".to_string()),
            RuntimeValue::Float(timestamp),
        ),
    ])
}

/// Extract an integer field from a Dict value by key name.
fn dict_get_int(pairs: &[(RuntimeValue, RuntimeValue)], key: &str) -> Option<i64> {
    for (k, v) in pairs {
        if k.as_string() == Some(key) {
            return v.as_int();
        }
    }
    None
}

/// Convert a datetime Dict to a NaiveDateTime.
fn dict_to_naive_datetime(
    pairs: &[(RuntimeValue, RuntimeValue)],
) -> Result<NaiveDateTime, RuntimeError> {
    let year = dict_get_int(pairs, "year")
        .ok_or_else(|| RuntimeError::type_error("datetime dict missing 'year' key"))?
        as i32;
    let month = dict_get_int(pairs, "month")
        .ok_or_else(|| RuntimeError::type_error("datetime dict missing 'month' key"))?
        as u32;
    let day = dict_get_int(pairs, "day")
        .ok_or_else(|| RuntimeError::type_error("datetime dict missing 'day' key"))?
        as u32;
    let hour = dict_get_int(pairs, "hour").unwrap_or(0) as u32;
    let minute = dict_get_int(pairs, "minute").unwrap_or(0) as u32;
    let second = dict_get_int(pairs, "second").unwrap_or(0) as u32;
    let microsecond = dict_get_int(pairs, "microsecond").unwrap_or(0) as u32;

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_micro_opt(hour, minute, second, microsecond))
        .ok_or_else(|| RuntimeError::type_error("datetime dict contains invalid date/time values"))
}

// ── Built-in functions ────────────────────────────────────────────────────────

fn builtin_datetime_now(_args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let now = Local::now();
    let naive = now.naive_local();
    let timestamp = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1_000_000.0;
    Ok(naive_datetime_to_dict(naive, timestamp))
}

fn builtin_datetime_utcnow(_args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let now = Utc::now();
    let naive = now.naive_utc();
    let timestamp = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1_000_000.0;
    Ok(naive_datetime_to_dict(naive, timestamp))
}

fn builtin_datetime_parse(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "datetime_parse() requires 2 arguments: datetime_parse(s, fmt)",
        ));
    }
    let s = args[0].as_string().ok_or_else(|| {
        RuntimeError::type_error("datetime_parse() first argument must be a string")
    })?;
    let fmt = args[1].as_string().ok_or_else(|| {
        RuntimeError::type_error("datetime_parse() second argument must be a string")
    })?;

    match NaiveDateTime::parse_from_str(s, fmt) {
        Ok(dt) => {
            // Compute timestamp assuming the naive time is in UTC
            let timestamp = Utc.from_utc_datetime(&dt).timestamp() as f64;
            Ok(naive_datetime_to_dict(dt, timestamp))
        }
        Err(_) => Ok(RuntimeValue::Null),
    }
}

fn builtin_datetime_format(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "datetime_format() requires 2 arguments: datetime_format(dt_dict, fmt)",
        ));
    }
    let pairs = match &args[0] {
        RuntimeValue::Dict(p) => p,
        _ => {
            return Err(RuntimeError::type_error(
                "datetime_format() first argument must be a datetime dict",
            ))
        }
    };
    let fmt = args[1].as_string().ok_or_else(|| {
        RuntimeError::type_error("datetime_format() second argument must be a string")
    })?;

    let dt = dict_to_naive_datetime(pairs)?;
    Ok(RuntimeValue::String(dt.format(fmt).to_string()))
}

fn builtin_datetime_add(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "datetime_add() requires 2 arguments: datetime_add(dt_dict, delta_dict)",
        ));
    }
    let dt_pairs = match &args[0] {
        RuntimeValue::Dict(p) => p,
        _ => {
            return Err(RuntimeError::type_error(
                "datetime_add() first argument must be a datetime dict",
            ))
        }
    };
    let delta_pairs = match &args[1] {
        RuntimeValue::Dict(p) => p,
        _ => {
            return Err(RuntimeError::type_error(
                "datetime_add() second argument must be a delta dict",
            ))
        }
    };

    let dt = dict_to_naive_datetime(dt_pairs)?;

    let days = dict_get_int(delta_pairs, "days").unwrap_or(0);
    let hours = dict_get_int(delta_pairs, "hours").unwrap_or(0);
    let minutes = dict_get_int(delta_pairs, "minutes").unwrap_or(0);
    let seconds = dict_get_int(delta_pairs, "seconds").unwrap_or(0);

    let total_seconds = days * 86_400 + hours * 3_600 + minutes * 60 + seconds;
    let duration = chrono::Duration::try_seconds(total_seconds)
        .ok_or_else(|| RuntimeError::type_error("datetime_add(): delta overflow"))?;

    let new_dt = dt
        .checked_add_signed(duration)
        .ok_or_else(|| RuntimeError::type_error("datetime_add(): result out of range"))?;

    let timestamp = Utc.from_utc_datetime(&new_dt).timestamp() as f64;
    Ok(naive_datetime_to_dict(new_dt, timestamp))
}

fn builtin_datetime_diff_seconds(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "datetime_diff_seconds() requires 2 arguments: datetime_diff_seconds(dt1, dt2)",
        ));
    }
    let p1 = match &args[0] {
        RuntimeValue::Dict(p) => p,
        _ => {
            return Err(RuntimeError::type_error(
                "datetime_diff_seconds() first argument must be a datetime dict",
            ))
        }
    };
    let p2 = match &args[1] {
        RuntimeValue::Dict(p) => p,
        _ => {
            return Err(RuntimeError::type_error(
                "datetime_diff_seconds() second argument must be a datetime dict",
            ))
        }
    };

    let dt1 = dict_to_naive_datetime(p1)?;
    let dt2 = dict_to_naive_datetime(p2)?;

    let diff = dt1.signed_duration_since(dt2);
    Ok(RuntimeValue::Float(
        diff.num_milliseconds() as f64 / 1_000.0,
    ))
}

fn builtin_timestamp_to_datetime(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let ts = args.first().and_then(|v| v.as_float()).ok_or_else(|| {
        RuntimeError::type_error("timestamp_to_datetime() requires a numeric argument")
    })?;

    let secs = ts.trunc() as i64;
    let micros = ((ts.fract().abs()) * 1_000_000.0).round() as u32;

    let dt = chrono::DateTime::from_timestamp(secs, micros * 1_000)
        .ok_or_else(|| RuntimeError::type_error("timestamp_to_datetime(): timestamp out of range"))?
        .naive_utc();

    Ok(naive_datetime_to_dict(dt, ts))
}

fn builtin_datetime_to_timestamp(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let pairs = match args.first() {
        Some(RuntimeValue::Dict(p)) => p,
        _ => {
            return Err(RuntimeError::type_error(
                "datetime_to_timestamp() requires a datetime dict argument",
            ))
        }
    };

    let dt = dict_to_naive_datetime(pairs)?;
    let timestamp = Utc.from_utc_datetime(&dt).timestamp() as f64;
    Ok(RuntimeValue::Float(timestamp))
}
