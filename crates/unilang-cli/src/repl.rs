// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang repl` — interactive read-eval-print loop.

use std::io::{self, BufRead, Write};
use unilang_common::source::SourceMap;
use unilang_runtime::error::ErrorKind;

/// Entry point called from main.
pub fn cmd_repl() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "UniLang REPL v{}  (type 'exit' or Ctrl+C to quit)",
        version
    );

    let stdin = io::stdin();
    let mut lines_iter = stdin.lock().lines();

    loop {
        // Primary prompt
        print!(">>> ");
        io::stdout().flush().ok();

        let first_line = match lines_iter.next() {
            Some(Ok(l)) => l,
            Some(Err(_)) => break,
            None => break, // EOF / Ctrl+D
        };

        let trimmed = first_line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        // Decide whether we need to read more lines (block detection).
        let mut input = first_line.clone();

        if needs_continuation(&first_line) {
            // Keep reading until a blank line is entered.
            loop {
                print!("... ");
                io::stdout().flush().ok();

                match lines_iter.next() {
                    Some(Ok(line)) => {
                        if line.trim().is_empty() {
                            break;
                        }
                        input.push('\n');
                        input.push_str(&line);
                    }
                    Some(Err(_)) | None => break,
                }
            }
        }

        if input.trim().is_empty() {
            continue;
        }

        eval_and_print(&input);
    }
}

/// Return true if the last non-empty line ends with `:` or `{`.
fn needs_continuation(line: &str) -> bool {
    let t = line.trim_end();
    t.ends_with(':') || t.ends_with('{')
}

/// Parse, analyze, compile, and run a snippet; print output or error.
fn eval_and_print(source: &str) {
    let mut source_map = SourceMap::new();
    let source_id = source_map.add("<repl>".to_string(), source.to_string());

    // Parse
    let (module, parse_diags) = unilang_parser::parse(source_id, source);
    if parse_diags.has_errors() {
        for d in parse_diags.diagnostics() {
            eprintln!("parse error: {}", d.message);
        }
        return;
    }

    // Semantic analysis
    let driver_funcs = unilang_drivers::default_registry().all_function_names();
    let (_result, sem_diags) =
        unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);
    if sem_diags.has_errors() {
        for d in sem_diags.diagnostics() {
            eprintln!("error: {}", d.message);
        }
        return;
    }

    // Codegen
    let bytecode = match unilang_codegen::compile(&module) {
        Ok(bc) => bc,
        Err(diags) => {
            for d in &diags {
                eprintln!("compile error: {}", d.message);
            }
            return;
        }
    };

    // Execute
    let mut vm = unilang_runtime::vm::VM::new();
    unilang_stdlib::register_builtins(&mut vm);
    let drivers = unilang_drivers::default_registry();
    drivers.register_all(&mut vm);

    match vm.run(&bytecode) {
        Ok(_) => {}
        Err(e) => {
            if e.kind != ErrorKind::Halt {
                eprintln!("runtime error: {}", e.message);
            }
        }
    }
}
