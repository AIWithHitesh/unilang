// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang CLI — the command-line interface for the UniLang compiler.
//!
//! Current commands:
//! - `unilang lex <file>` — tokenize a .uniL file and print the token stream
//! - `unilang parse <file>` — parse a .uniL file and print the AST (TODO)

use clap::{Parser, Subcommand};
use std::fs;
use std::process;
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lex { file } => cmd_lex(&file),
        Commands::Parse { file } => cmd_parse(&file),
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
    eprintln!(
        "Parser not yet implemented. Use `unilang lex {}` to tokenize.",
        path
    );
    process::exit(1);
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
