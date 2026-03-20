// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! LSP backend implementing the `LanguageServer` trait for UniLang.

use std::collections::HashMap;
use std::sync::Arc;

use ropey::Rope;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics;
use crate::hover;

/// Document state stored per open file.
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub rope: Rope,
    pub version: i32,
}

/// The UniLang LSP backend.
pub struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, DocumentState>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publish diagnostics for a given document URI.
    async fn publish_diagnostics(&self, uri: Url, text: &str, version: i32) {
        let diags = diagnostics::generate_diagnostics(text);
        self.client
            .publish_diagnostics(uri, diags, Some(version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "UniLang LSP server initializing...")
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("unilang".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: Some(false),
                        },
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "unilang-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "UniLang LSP server initialized.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "UniLang LSP server shutting down.")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;

        {
            let mut docs = self.documents.write().await;
            docs.insert(
                uri.clone(),
                DocumentState {
                    rope: Rope::from_str(&text),
                    version,
                },
            );
        }

        self.publish_diagnostics(uri, &text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        // We use full sync, so there is exactly one change with the full text.
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text.clone();

            {
                let mut docs = self.documents.write().await;
                docs.insert(
                    uri.clone(),
                    DocumentState {
                        rope: Rope::from_str(&text),
                        version,
                    },
                );
            }

            self.publish_diagnostics(uri, &text, version).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        let (text, version) = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc) => (doc.rope.to_string(), doc.version),
                None => return,
            }
        };

        self.publish_diagnostics(uri, &text, version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.write().await.remove(&uri);
        // Clear diagnostics for closed files.
        self.client
            .publish_diagnostics(uri, Vec::new(), None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let doc = match docs.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = doc.rope.to_string();
        Ok(hover::get_hover_info(&text, position))
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.read().await;
        let doc = match docs.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = doc.rope.to_string();
        let items = completion::get_completions(&text, position);
        Ok(Some(CompletionResponse::Array(items)))
    }
}
