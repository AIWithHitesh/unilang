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

mod build;
mod formatter;
mod init;
mod linter;
mod pack;
mod project;
mod repl;
mod test_runner;

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
    /// Create a new UniLang project (interactive wizard or flags).
    New {
        /// Project name.
        name: Option<String>,
        /// Project type: web | app | cli.
        #[arg(long)]
        r#type: Option<String>,
        /// Comma-separated drivers: sqlite,redis,mysql,...
        #[arg(long)]
        deps: Option<String>,
        /// Parent directory (default: current dir).
        #[arg(long)]
        path: Option<String>,
        /// Skip git init.
        #[arg(long)]
        no_git: bool,
        /// Accept defaults, skip wizard.
        #[arg(short, long)]
        yes: bool,
    },
    /// Driver management commands.
    Driver {
        #[command(subcommand)]
        action: DriverAction,
    },
    /// Discover and run test_ functions in .uniL files.
    Test {
        /// File or directory to scan (default: current dir).
        path: Option<String>,
    },
    /// Format .uniL source files.
    Fmt {
        /// File or directory to format (default: current dir).
        path: Option<String>,
        /// Write formatted output back to files instead of stdout.
        #[arg(long)]
        write: bool,
    },
    /// Run linting / style checks on .uniL files.
    Lint {
        /// File or directory to lint (default: current dir).
        path: Option<String>,
    },
    /// Interactive read-eval-print loop.
    Repl,
    /// Bundle project into a .uniLpkg archive.
    Pack {
        /// Output file path (default: <name>-<version>.uniLpkg).
        #[arg(long)]
        out: Option<String>,
    },
    /// Compile a .uniL file without executing it.
    Build {
        /// Path to the .uniL source file.
        file: String,
        /// Enable incremental compilation cache.
        #[arg(long)]
        incremental: bool,
    },
    /// Show or initialise project configuration.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

/// Sub-commands for `unilang config`.
#[derive(Subcommand)]
enum ConfigAction {
    /// Pretty-print the parsed unilang.toml.
    Show,
    /// Scaffold a new unilang.toml in the current directory.
    Init,
}

/// Sub-commands for `unilang driver`.
#[derive(Subcommand)]
enum DriverAction {
    /// List all registered drivers with their metadata and exported functions.
    List,
    /// Generate a new community driver template file.
    New {
        /// Driver name (e.g. "cassandra" generates cassandra.rs).
        name: String,
        /// Output directory (default: current directory).
        #[arg(long)]
        out: Option<String>,
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
        Commands::New {
            name,
            r#type,
            deps,
            path,
            no_git,
            yes,
        } => init::cmd_new(name, r#type, deps, path, no_git, yes),
        Commands::Driver { action } => match action {
            DriverAction::List => cmd_driver_list(),
            DriverAction::New { name, out } => cmd_driver_new(&name, out.as_deref()),
        },
        Commands::Test { path } => {
            let code = test_runner::cmd_test(path.as_deref());
            process::exit(code);
        }
        Commands::Fmt { path, write } => {
            formatter::cmd_fmt(path.as_deref(), write);
        }
        Commands::Lint { path } => {
            let code = linter::cmd_lint(path.as_deref());
            process::exit(code);
        }
        Commands::Repl => repl::cmd_repl(),
        Commands::Pack { out } => pack::cmd_pack(out.as_deref()),
        Commands::Build { file, incremental } => build::cmd_build(&file, incremental),
        Commands::Config { action } => match action {
            ConfigAction::Show => project::cmd_config_show(),
            ConfigAction::Init => project::cmd_config_init(),
        },
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

    // Collect driver function names so the semantic analyzer recognises them.
    let driver_funcs = unilang_drivers::default_registry().all_function_names();

    // Semantic analysis
    let (_result, sem_diags) =
        unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);

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

    // 2. Semantic analysis — inject driver function names so community-driver
    //    calls are not flagged as "undefined variable".
    let driver_funcs = unilang_drivers::default_registry().all_function_names();
    let (_result, sem_diags) =
        unilang_semantic::analyze_with_extra_builtins(&module, &driver_funcs);

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

    // 4. Create VM, register stdlib builtins + all drivers, and execute
    let mut vm = unilang_runtime::vm::VM::new();
    unilang_stdlib::register_builtins(&mut vm);
    let drivers = unilang_drivers::default_registry();
    drivers.register_all(&mut vm);

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

// ── Driver management commands ────────────────────────────────────────────────

fn cmd_driver_list() {
    let registry = unilang_drivers::default_registry();

    if registry.drivers().is_empty() {
        println!("No drivers registered.");
        return;
    }

    println!(
        "{:<22} {:<10} {:<20} FUNCTIONS",
        "NAME", "VERSION", "CATEGORY"
    );
    println!("{}", "-".repeat(90));

    for driver in registry.drivers() {
        let funcs = driver.exported_functions();
        let func_count = funcs.len();
        let func_list = funcs.join(", ");
        println!(
            "{:<22} {:<10} {:<20} ({}) {}",
            driver.name(),
            driver.version(),
            format!("{:?}", driver.category()),
            func_count,
            func_list
        );
    }

    println!();
    println!("Total: {} driver(s) registered.", registry.drivers().len());
}

fn cmd_driver_new(name: &str, out: Option<&str>) {
    // Capitalize first letter for the struct name.
    let struct_name = {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    let template = format!(
        r#"// TEMPLATE: {name} driver for UniLang
// Copy this file to crates/unilang-drivers/src/community/{name}.rs
// Then run `cargo build` — the driver will be auto-discovered.
//
// No other changes needed!

use std::sync::{{Arc, Mutex}};
use unilang_runtime::error::RuntimeError;
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;
use crate::{{DriverCategory, UniLangDriver}};

pub struct {struct_name} {{
    // Add your connection state here, wrapped in Arc<Mutex<Option<...>>>
    // so it can be safely shared across multiple registered closures.
    _state: Arc<Mutex<Option<()>>>,
}}

impl {struct_name} {{
    pub fn new() -> Self {{
        Self {{ _state: Arc::new(Mutex::new(None)) }}
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{ Self::new() }}
}}

impl UniLangDriver for {struct_name} {{
    fn name(&self)        -> &str {{ "{name}" }}
    fn version(&self)     -> &str {{ "0.1.0" }}
    fn description(&self) -> &str {{ "{struct_name} driver for UniLang" }}
    fn category(&self)    -> DriverCategory {{ DriverCategory::Other }}

    fn exported_functions(&self) -> &'static [&'static str] {{
        &["{name}_connect", "{name}_query", "{name}_close"]
    }}

    fn register(&self, vm: &mut VM) {{
        let state = Arc::clone(&self._state);
        vm.register_builtin("{name}_connect", move |args| {{
            let _url = match args.first() {{
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err(RuntimeError::type_error(
                    "{name}_connect(url): expected string".to_string()
                )),
            }};
            // TODO: open connection, store in state
            *state.lock().unwrap() = Some(());
            Ok(RuntimeValue::Bool(true))
        }});

        let state = Arc::clone(&self._state);
        vm.register_builtin("{name}_query", move |args| {{
            let guard = state.lock().unwrap();
            if guard.is_none() {{
                return Err(RuntimeError::type_error(
                    "{name}_query: not connected — call {name}_connect first".to_string()
                ));
            }}
            let _sql = match args.first() {{
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err(RuntimeError::type_error(
                    "{name}_query(sql): expected string".to_string()
                )),
            }};
            // TODO: execute query, convert rows to Vec<RuntimeValue>
            Ok(RuntimeValue::List(vec![]))
        }});

        let state = Arc::clone(&self._state);
        vm.register_builtin("{name}_close", move |_args| {{
            *state.lock().unwrap() = None;
            Ok(RuntimeValue::Bool(true))
        }});
    }}
}}
"#,
        name = name,
        struct_name = struct_name,
    );

    let out_dir = out.unwrap_or(".");
    let file_path = std::path::Path::new(out_dir).join(format!("{}.rs", name));

    match fs::write(&file_path, &template) {
        Ok(_) => {
            println!("Created driver template: {}", file_path.display());
            println!();
            println!("Next steps:");
            println!(
                "  1. Move the file to crates/unilang-drivers/src/community/{}.rs",
                name
            );
            println!("  2. Implement the TODO sections");
            println!("  3. Run `cargo build` — the driver is auto-discovered");
            println!(
                "  4. Test with a .uniL script calling {}_connect / {}_query / {}_close",
                name, name, name
            );
        }
        Err(e) => {
            eprintln!("error: cannot write '{}': {}", file_path.display(), e);
            process::exit(1);
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
