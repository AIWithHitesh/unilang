// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! JSON built-in functions: json_encode, json_decode.

use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

/// Register JSON built-in functions.
pub fn register_all(vm: &mut VM) {
    vm.register_builtin("json_encode", builtin_json_encode);
    vm.register_builtin("json_decode", builtin_json_decode);
    // Aliases used by server.uniL
    vm.register_builtin("to_json",   builtin_json_encode);
    vm.register_builtin("from_json", builtin_json_decode);
}

// ── Encode ────────────────────────────────────────────────────

fn builtin_json_encode(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let val = args
        .first()
        .ok_or_else(|| RuntimeError::type_error("json_encode() requires 1 argument"))?;
    Ok(RuntimeValue::String(encode_value(val)))
}

fn encode_value(val: &RuntimeValue) -> String {
    match val {
        RuntimeValue::Null => "null".to_string(),
        RuntimeValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        RuntimeValue::Int(n) => n.to_string(),
        RuntimeValue::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        RuntimeValue::String(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for ch in s.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if (c as u32) < 0x20 => {
                        out.push_str(&format!("\\u{:04x}", c as u32));
                    }
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        RuntimeValue::List(items) => {
            let inner = items.iter().map(encode_value).collect::<Vec<_>>().join(",");
            format!("[{}]", inner)
        }
        RuntimeValue::Dict(pairs) => {
            let inner = pairs
                .iter()
                .map(|(k, v)| format!("{}:{}", encode_value(k), encode_value(v)))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{}}}", inner)
        }
        _ => "null".to_string(),
    }
}

// ── Decode ────────────────────────────────────────────────────

fn builtin_json_decode(args: &[RuntimeValue]) -> Result<RuntimeValue, RuntimeError> {
    let s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| RuntimeError::type_error("json_decode() requires a string argument"))?
        .to_string();
    let mut p = JsonParser::new(&s);
    p.parse_value()
        .ok_or_else(|| RuntimeError::type_error(format!("json_decode(): invalid JSON")))
}

struct JsonParser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(s: &'a str) -> Self {
        Self { src: s.as_bytes(), pos: 0 }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && (self.src[self.pos] as char).is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn eat(&mut self) -> Option<u8> {
        let b = self.src.get(self.pos).copied();
        if b.is_some() { self.pos += 1; }
        b
    }

    fn expect_bytes(&mut self, bytes: &[u8]) -> bool {
        if self.src[self.pos..].starts_with(bytes) {
            self.pos += bytes.len();
            true
        } else {
            false
        }
    }

    fn parse_value(&mut self) -> Option<RuntimeValue> {
        self.skip_ws();
        match self.peek()? {
            b'"' => self.parse_string().map(RuntimeValue::String),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b't' => {
                if self.expect_bytes(b"true") { Some(RuntimeValue::Bool(true)) } else { None }
            }
            b'f' => {
                if self.expect_bytes(b"false") { Some(RuntimeValue::Bool(false)) } else { None }
            }
            b'n' => {
                if self.expect_bytes(b"null") { Some(RuntimeValue::Null) } else { None }
            }
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => None,
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        // consume opening "
        self.eat()?; // "
        let mut s = String::new();
        loop {
            match self.eat()? {
                b'"' => break,
                b'\\' => {
                    match self.eat()? {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'b' => s.push('\x08'),
                        b'f' => s.push('\x0C'),
                        b'u' => {
                            // Read 4 hex digits
                            let mut hex = String::new();
                            for _ in 0..4 {
                                hex.push(self.eat()? as char);
                            }
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(c) = char::from_u32(code) {
                                    s.push(c);
                                }
                            }
                        }
                        other => s.push(other as char),
                    }
                }
                b => s.push(b as char),
            }
        }
        Some(s)
    }

    fn parse_number(&mut self) -> Option<RuntimeValue> {
        let start = self.pos;
        let mut is_float = false;
        if self.peek() == Some(b'-') { self.pos += 1; }
        while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) { self.pos += 1; }
            while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
        }
        let slice = std::str::from_utf8(&self.src[start..self.pos]).ok()?;
        if is_float {
            slice.parse::<f64>().ok().map(RuntimeValue::Float)
        } else {
            slice.parse::<i64>().ok().map(RuntimeValue::Int)
        }
    }

    fn parse_array(&mut self) -> Option<RuntimeValue> {
        self.eat()?; // [
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.eat();
            return Some(RuntimeValue::List(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek()? {
                b',' => { self.eat(); }
                b']' => { self.eat(); break; }
                _ => return None,
            }
        }
        Some(RuntimeValue::List(items))
    }

    fn parse_object(&mut self) -> Option<RuntimeValue> {
        self.eat()?; // {
        self.skip_ws();
        let mut pairs = Vec::new();
        if self.peek() == Some(b'}') {
            self.eat();
            return Some(RuntimeValue::Dict(pairs));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string().map(RuntimeValue::String)?;
            self.skip_ws();
            if self.eat()? != b':' { return None; }
            let val = self.parse_value()?;
            pairs.push((key, val));
            self.skip_ws();
            match self.peek()? {
                b',' => { self.eat(); }
                b'}' => { self.eat(); break; }
                _ => return None,
            }
        }
        Some(RuntimeValue::Dict(pairs))
    }
}
