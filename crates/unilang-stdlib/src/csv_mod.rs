// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! CSV built-in functions: csv_read, csv_read_header, csv_write,
//! csv_parse, csv_stringify.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register CSV built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("csv_read", builtin_csv_read);
    vm.register_builtin("csv_read_header", builtin_csv_read_header);
    vm.register_builtin("csv_write", builtin_csv_write);
    vm.register_builtin("csv_parse", builtin_csv_parse);
    vm.register_builtin("csv_stringify", builtin_csv_stringify);
}

// ── Built-in functions ────────────────────────────────────────────────────────

fn builtin_csv_read(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("csv_read() requires a string path argument"))?;
    let content = std::fs::read_to_string(path).map_err(|e| {
        RuntimeError::type_error(format!("csv_read(): cannot read file {:?}: {}", path, e))
    })?;
    csv_parse_impl(&content)
}

fn builtin_csv_read_header(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let path = args.first().and_then(|v| v.as_string()).ok_or_else(|| {
        RuntimeError::type_error("csv_read_header() requires a string path argument")
    })?;
    let content = std::fs::read_to_string(path).map_err(|e| {
        RuntimeError::type_error(format!(
            "csv_read_header(): cannot read file {:?}: {}",
            path, e
        ))
    })?;
    csv_parse_with_headers_impl(&content)
}

fn builtin_csv_write(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::type_error(
            "csv_write() requires two arguments: path and rows",
        ));
    }
    let path = args[0]
        .as_string()
        .ok_or_else(|| RuntimeError::type_error("csv_write(): path must be a string"))?;
    let rows = match &args[1] {
        RuntimeValue::List(rows) => rows,
        _ => {
            return Err(RuntimeError::type_error(
                "csv_write(): rows must be a list of lists",
            ))
        }
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for row_val in rows {
        match row_val {
            RuntimeValue::List(cells) => {
                let record: Vec<String> = cells
                    .iter()
                    .map(|c| match c {
                        RuntimeValue::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect();
                wtr.write_record(&record).map_err(|e| {
                    RuntimeError::type_error(format!("csv_write(): write error: {}", e))
                })?;
            }
            _ => {
                return Err(RuntimeError::type_error(
                    "csv_write(): each row must be a list",
                ))
            }
        }
    }
    let bytes = wtr
        .into_inner()
        .map_err(|e| RuntimeError::type_error(format!("csv_write(): flush error: {}", e)))?;

    std::fs::write(path, &bytes).map_err(|e| {
        RuntimeError::type_error(format!("csv_write(): cannot write file {:?}: {}", path, e))
    })?;

    Ok(RuntimeValue::Bool(true))
}

fn builtin_csv_parse(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let text = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("csv_parse() requires a string argument"))?;
    csv_parse_impl(text)
}

fn builtin_csv_stringify(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let rows = match args.first() {
        Some(RuntimeValue::List(rows)) => rows,
        _ => {
            return Err(RuntimeError::type_error(
                "csv_stringify() requires a list of lists argument",
            ))
        }
    };

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut buf);
        for row_val in rows {
            match row_val {
                RuntimeValue::List(cells) => {
                    let record: Vec<String> = cells
                        .iter()
                        .map(|c| match c {
                            RuntimeValue::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect();
                    wtr.write_record(&record).map_err(|e| {
                        RuntimeError::type_error(format!("csv_stringify(): write error: {}", e))
                    })?;
                }
                _ => {
                    return Err(RuntimeError::type_error(
                        "csv_stringify(): each row must be a list",
                    ))
                }
            }
        }
        wtr.flush().map_err(|e| {
            RuntimeError::type_error(format!("csv_stringify(): flush error: {}", e))
        })?;
    }

    let s = String::from_utf8(buf).map_err(|e| {
        RuntimeError::type_error(format!("csv_stringify(): output is not valid UTF-8: {}", e))
    })?;
    Ok(RuntimeValue::String(s))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn csv_parse_impl(text: &str) -> Result<RuntimeValue, RuntimeError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());

    let mut rows: Vec<RuntimeValue> = Vec::new();
    for result in reader.records() {
        let record = result
            .map_err(|e| RuntimeError::type_error(format!("csv_parse(): parse error: {}", e)))?;
        let cells: Vec<RuntimeValue> = record
            .iter()
            .map(|cell| RuntimeValue::String(cell.to_string()))
            .collect();
        rows.push(RuntimeValue::List(cells));
    }
    Ok(RuntimeValue::List(rows))
}

fn csv_parse_with_headers_impl(text: &str) -> Result<RuntimeValue, RuntimeError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| RuntimeError::type_error(format!("csv_read_header(): header error: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut rows: Vec<RuntimeValue> = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| {
            RuntimeError::type_error(format!("csv_read_header(): parse error: {}", e))
        })?;
        let pairs: Vec<(RuntimeValue, RuntimeValue)> = headers
            .iter()
            .zip(record.iter())
            .map(|(k, v)| {
                (
                    RuntimeValue::String(k.clone()),
                    RuntimeValue::String(v.to_string()),
                )
            })
            .collect();
        rows.push(RuntimeValue::Dict(pairs));
    }
    Ok(RuntimeValue::List(rows))
}
