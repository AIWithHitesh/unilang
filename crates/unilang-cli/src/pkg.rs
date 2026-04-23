// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! `unilang pkg` sub-commands — package manager CLI handlers.

use std::process;
use unilang_pkg::PkgError;

/// `unilang pkg install <package>[@version] [--dir <dir>]`
pub fn cmd_install(package: &str, dir: &str) {
    // Split `name@version` if present.
    let (name, version) = if let Some(at) = package.find('@') {
        (&package[..at], Some(&package[at + 1..]))
    } else {
        (package, None)
    };

    eprintln!("installing {} …", package);
    match unilang_pkg::install(name, version, dir) {
        Ok(()) => {}
        Err(PkgError::NetworkError(msg)) => {
            eprintln!("error: offline or registry unreachable — {}", msg);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

/// `unilang pkg publish [--dir <dir>] --token <token>`
pub fn cmd_publish(dir: &str, token: &str) {
    let manifest = match unilang_pkg::read_manifest(dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: cannot read manifest — {}", e);
            process::exit(1);
        }
    };

    eprintln!("publishing {}@{} …", manifest.name, manifest.version);
    match unilang_pkg::publish(&manifest, dir, token) {
        Ok(url) => println!("published: {}", url),
        Err(PkgError::NetworkError(msg)) => {
            eprintln!("error: offline or registry unreachable — {}", msg);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

/// `unilang pkg search <query>`
pub fn cmd_search(query: &str) {
    match unilang_pkg::search(query) {
        Ok(results) => {
            if results.is_empty() {
                println!("No packages found for '{}'.", query);
                return;
            }
            println!(
                "{:<30} {:<12} {:<12} DESCRIPTION",
                "NAME", "LATEST", "DOWNLOADS"
            );
            println!("{}", "-".repeat(90));
            for pkg in &results {
                println!(
                    "{:<30} {:<12} {:<12} {}",
                    pkg.name, pkg.latest, pkg.downloads, pkg.description
                );
            }
            println!("\n{} package(s) found.", results.len());
        }
        Err(PkgError::NetworkError(msg)) => {
            eprintln!("error: offline or registry unreachable — {}", msg);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

/// `unilang pkg list [--dir <dir>]`
pub fn cmd_list(dir: &str) {
    match unilang_pkg::list_installed(dir) {
        Ok(packages) => {
            if packages.is_empty() {
                println!("No packages installed in '{}'.", dir);
                return;
            }
            println!("{:<30} {:<12} DESCRIPTION", "NAME", "VERSION");
            println!("{}", "-".repeat(70));
            for pkg in &packages {
                println!("{:<30} {:<12} {}", pkg.name, pkg.version, pkg.description);
            }
            println!("\n{} package(s) installed.", packages.len());
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

/// `unilang pkg init <name> [--dir <dir>]`
pub fn cmd_init(name: &str, dir: &str) {
    match unilang_pkg::init_manifest(name, dir) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
