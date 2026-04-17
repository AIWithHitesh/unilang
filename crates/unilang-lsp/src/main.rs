// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! # UniLang Language Server
//!
//! LSP server for `.uniL` files, providing real-time diagnostics,
//! hover information, and auto-completion for all IDEs.

mod backend;
mod completion;
mod definition;
mod diagnostics;
mod formatting;
mod hover;

use backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    eprintln!("UniLang LSP server starting...");

    Server::new(stdin, stdout, socket).serve(service).await;
}
