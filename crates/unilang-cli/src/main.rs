// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang CLI — the command-line interface for the UniLang compiler.
//!
//! Commands:
//! - `unilang lex <file>`     — tokenize a .uniL file and print the token stream
//! - `unilang parse <file>`   — parse a .uniL file and print the AST
//! - `unilang check <file>`   — lex, parse, and run semantic analysis
//! - `unilang compile <file>` — lex, parse, compile, and print bytecode disassembly
//! - `unilang run <file>`     — full pipeline: lex, parse, analyze, compile, execute

use clap::{Parser, Subcommand};
use std::fs;
use std::process;
use unilang_common::error::Severity;
use unilang_common::source::SourceMap;
use unilang_lexer::Lexer;

#[derive(Parser)]
#[command(name = "unilang")]
#[command(about = "UniLang compiler — a unified Python + Java language")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Tokenize a .uniL file and print the token stream.
    Lex {
        /// Path to the .uniL source file.
        file: String,
    },
    /// Parse a .uniL file and print the AST.
    Parse {
        /// Path to the .uniL source file.
        file: String,
    },
    /// Lex, parse, and run semantic analysis (report diagnostics only).
    Check {
        /// Path to the .uniL source file.
        file: String,
    },
    /// Lex, parse, compile, and print bytecode disassembly.
    Compile {
        /// Path to the .uniL source file.
        file: String,
    },
    /// Full pipeline: lex, parse, analyze, compile, and execute.
    Run {
        /// Path to the .uniL source file.
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lex { file } => cmd_lex(&file),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Check { file } => cmd_check(&file),
        Commands::Compile { file } => cmd_compile(&file),
        Commands::Run { file } => cmd_run(&file),
    }
}

fn cmd_lex(path: &str) {
    let source = read_file(path);
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path.to_string(), source.clone());

    let lexer = Lexer::new(source_id, &source);
    let (tokens, diagnostics) = lexer.tokenize();

    let file = source_map.get(source_id);

    for token in &tokens {
        let lc = file.line_col(token.span.start);
        let text = if !token.span.is_empty() {
            file.slice(token.span)
        } else {
            ""
        };
        println!(
            "{:>4}:{:<3}  {:20} {:?}",
            lc.line,
            lc.col,
            format!("{}", token.kind),
            text
        );
    }

    if diagnostics.has_errors() {
        eprintln!();
        for d in diagnostics.diagnostics() {
            let code = d.code.as_deref().unwrap_or("???");
            eprintln!("error[{}]: {}", code, d.message);
            for label in &d.labels {
                let lc = file.line_col(label.span.start);
                eprintln!("  --> {}:{}:{}: {}", path, lc.line, lc.col, label.message);
            }
            for note in &d.notes {
                eprintln!("  = note: {}", note);
            }
        }
        eprintln!();
        eprintln!("{} error(s) found.", diagnostics.error_count());
        process::exit(1);
    }

    eprintln!("{} tokens emitted.", tokens.len());
}

fn cmd_parse(path: &str) {
    let source = read_file(path);
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path.to_string(), source.clone());

    let (module, diags) = unilang_parser::parse(source_id, &source);

    if diags.has_errors() {
        print_diagnostics(path, &source_map, source_id, &diags);
        process::exit(1);
    }

    println!("{:#?}", module);
    eprintln!("{} statement(s) parsed.", module.statements.len());
}

fn cmd_check(path: &str) {
    let source = read_file(path);
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path.to_string(), source.clone());

    // Lex + parse
    let (module, parse_diags) = unilang_parser::parse(source_id, &source);
    if parse_diags.has_errors() {
        print_diagnostics(path, &source_map, source_id, &parse_diags);
        process::exit(1);
    }

    // Semantic analysis
    let (_result, sem_diags) = unilang_semantic::analyze(&module);

    if sem_diags.has_errors() {
        print_diagnostics(path, &source_map, source_id, &sem_diags);
        eprintln!(
            "{} error(s), {} warning(s).",
            sem_diags.error_count(),
            sem_diags
                .diagnostics()
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count()
        );
        process::exit(1);
    }

    let warnings = sem_diags
        .diagnostics()
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    if warnings > 0 {
        print_diagnostics(path, &source_map, source_id, &sem_diags);
    }

    eprintln!("No errors. {} warning(s).", warnings);
}

fn cmd_compile(path: &str) {
    let source = read_file(path);
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path.to_string(), source.clone());

    // Lex + parse
    let (module, parse_diags) = unilang_parser::parse(source_id, &source);
    if parse_diags.has_errors() {
        print_diagnostics(path, &source_map, source_id, &parse_diags);
        process::exit(1);
    }

    // Compile
    match unilang_codegen::compile(&module) {
        Ok(bytecode) => {
            println!("=== Bytecode Disassembly ===");
            println!("--- Instructions ({}) ---", bytecode.instructions.len());
            for (i, op) in bytecode.instructions.iter().enumerate() {
                println!("  {:04}  {:?}", i, op);
            }
            println!("--- Functions ({}) ---", bytecode.functions.len());
            for (i, func) in bytecode.functions.iter().enumerate() {
                println!(
                    "  [{}] {} ({} params, {} locals)",
                    i,
                    func.name,
                    func.params.len(),
                    func.local_count
                );
                for (j, op) in func.code.iter().enumerate() {
                    println!("    {:04}  {:?}", j, op);
                }
            }
            println!("--- Classes ({}) ---", bytecode.classes.len());
            for (i, cls) in bytecode.classes.iter().enumerate() {
                println!(
                    "  [{}] {} (methods: {:?}, fields: {:?})",
                    i, cls.name, cls.methods, cls.fields
                );
            }
        }
        Err(diags) => {
            for d in &diags {
                let severity = match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Hint => "hint",
                };
                eprintln!("{}: {}", severity, d.message);
            }
            process::exit(1);
        }
    }
}

fn cmd_run(path: &str) {
    let source = read_file(path);
    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path.to_string(), source.clone());

    // 1. Lex + parse
    let (module, parse_diags) = unilang_parser::parse(source_id, &source);
    if parse_diags.has_errors() {
        print_diagnostics(path, &source_map, source_id, &parse_diags);
        process::exit(1);
    }

    // 2. Semantic analysis
    let (_result, sem_diags) = unilang_semantic::analyze(&module);

    // Print warnings
    for d in sem_diags.diagnostics() {
        if d.severity == Severity::Warning {
            let file = source_map.get(source_id);
            if let Some(label) = d.labels.first() {
                let lc = file.line_col(label.span.start);
                eprintln!("warning: {}:{}:{}: {}", path, lc.line, lc.col, d.message);
            } else {
                eprintln!("warning: {}", d.message);
            }
        }
    }

    if sem_diags.has_errors() {
        print_diagnostics(path, &source_map, source_id, &sem_diags);
        process::exit(1);
    }

    // 3. Compile to bytecode
    let bytecode = match unilang_codegen::compile(&module) {
        Ok(bc) => bc,
        Err(diags) => {
            for d in &diags {
                let severity = match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Hint => "hint",
                };
                eprintln!("{}: {}", severity, d.message);
            }
            process::exit(1);
        }
    };

    // 4. Create VM, register stdlib builtins, and execute
    let mut vm = unilang_runtime::vm::VM::new();
    unilang_stdlib::register_builtins(&mut vm);

    match vm.run(&bytecode) {
        Ok(_) => {}
        Err(e) => {
            if e.kind != unilang_runtime::error::ErrorKind::Halt {
                eprintln!("runtime error: {}", e.message);
                process::exit(1);
            }
        }
    }
}

fn print_diagnostics(
    path: &str,
    source_map: &SourceMap,
    source_id: unilang_common::span::SourceId,
    diags: &unilang_common::error::DiagnosticBag,
) {
    let file = source_map.get(source_id);
    for d in diags.diagnostics() {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        };
        let code = d.code.as_deref().unwrap_or("");
        if code.is_empty() {
            eprintln!("{}: {}", severity, d.message);
        } else {
            eprintln!("{}[{}]: {}", severity, code, d.message);
        }
        for label in &d.labels {
            let lc = file.line_col(label.span.start);
            eprintln!("  --> {}:{}:{}: {}", path, lc.line, lc.col, label.message);
        }
        for note in &d.notes {
            eprintln!("  = note: {}", note);
        }
    }
}

fn read_file(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", path, e);
            process::exit(1);
        }
    }
}
