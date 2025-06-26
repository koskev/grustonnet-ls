use std::sync::{Arc, RwLock};

use anyhow::{Result, anyhow};
use lsp_server::{Message, Notification, ResponseError};
use lsp_types::{
    CompletionList, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticParams, DocumentDiagnosticReportResult, PublishDiagnosticsParams,
    RelatedFullDocumentDiagnosticReport, ServerCapabilities, TextDocumentSyncKind,
    TextDocumentSyncOptions, Uri,
    notification::{Notification as NotifictionTrait, PublishDiagnostics},
};
use ropey::Rope;

use crate::{
    cache::Cache,
    completion::{
        Completion, global::GlobalCompletion, keyword::KeywordCompletion, local::LocalCompletion,
    },
    cst::completion::{CompletionInfo, CompletionType},
    diagnostics::{Diagnostics, eval::EvalDiagnostics, lint::LintDiagnostics},
    server::{
        config::Configuration,
        server::{LSPConnection, LSPResponse, LSPServer},
    },
};

#[derive(Default)]
pub struct JsonnetServer {
    pub cache: Cache,

    pub connection: LSPConnection,

    pub configuration: Arc<RwLock<Configuration>>,
}

impl JsonnetServer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn get_diagnostics(&self, filename: &str) -> Vec<Diagnostic> {
        let mut items = vec![];
        let config = self.configuration.read().unwrap().clone();
        if config.diagnostics.enable_eval {
            let diags = EvalDiagnostics::new(&self.cache).diagnostics(filename);
            items.extend(diags);
        }
        if config.diagnostics.enable_lint {
            let diags = LintDiagnostics::new(&self.cache).diagnostics(filename);
            items.extend(diags);
        }
        return items;
    }

    fn publish_diagnostics(&self, uri: Uri) {
        self.connection
            .send(Message::Notification(Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params: serde_json::to_value(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics: self.get_diagnostics(uri.as_str()),
                    version: None,
                })
                .unwrap(),
            }))
            .unwrap();
    }
}

impl LSPServer for JsonnetServer {
    fn connection(&self) -> &LSPConnection {
        &self.connection
    }
    fn get_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    ..Default::default()
                },
            )),

            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![".".into()]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn did_change_configuration(&self, params: DidChangeConfigurationParams) -> Result<()> {
        *self.configuration.write().unwrap() = Configuration::try_from(params)?;
        Ok(())
    }

    fn did_change_text(&self, params: DidChangeTextDocumentParams) -> Result<()> {
        for change in params.content_changes {
            let current_text = match self.cache.get_document(params.text_document.uri.as_str()) {
                Some(doc) => doc,
                None => return Err(anyhow!("Unable to find document in cache!")),
            };

            let range = match change.range {
                Some(r) => r,
                None => return Err(anyhow!("Got change params without range")),
            };
            let mut rope = Rope::from_str(&current_text.content);
            let idx_start =
                rope.line_to_char(range.start.line as usize) + range.start.character as usize;
            let idx_end = rope.line_to_char(range.end.line as usize) + range.end.character as usize;
            rope.remove(idx_start..idx_end);
            rope.insert(idx_start, &change.text);
            self.cache
                .update_content(params.text_document.uri.as_str(), rope.to_string().as_str());
            self.publish_diagnostics(params.text_document.uri.clone());
        }
        Ok(())
    }

    fn document_diagnostics(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<LSPResponse, ResponseError> {
        Ok(
            DocumentDiagnosticReportResult::Report(lsp_types::DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport {
                    full_document_diagnostic_report: lsp_types::FullDocumentDiagnosticReport {
                        items: self.get_diagnostics(params.text_document.uri.as_str()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .into(),
        )
    }

    fn did_open(&self, params: DidOpenTextDocumentParams) -> Result<()> {
        self.cache.update_content(
            params.text_document.uri.as_str(),
            &params.text_document.text,
        );
        self.publish_diagnostics(params.text_document.uri.clone());

        Ok(())
    }

    fn completion(&self, params: CompletionParams) -> Result<LSPResponse, ResponseError> {
        let doc = self
            .cache
            .get_document(params.text_document_position.text_document.uri.as_str())
            .unwrap();
        let completion_info =
            CompletionInfo::new(&doc.content, params.text_document_position.position.into());

        let mut lists = vec![];

        let config = self.configuration.read().unwrap().clone();
        match completion_info.completion_type {
            CompletionType::Global => {
                // Global completion
                if config.completion.enable_global {
                    let global_completion = GlobalCompletion::new(&self.cache);
                    lists.push(global_completion.complete(
                        completion_info.pos.clone(),
                        params.text_document_position.text_document.uri.as_str(),
                    ));
                }
                // Keyword completion
                if config.completion.enable_keywords {
                    let keyword_completion = KeywordCompletion::new(&self.cache);
                    lists.push(keyword_completion.complete(
                        completion_info.pos.clone(),
                        params.text_document_position.text_document.uri.as_str(),
                    ));
                }
            }
            CompletionType::Local => {
                if config.completion.enable_local {
                    let local_completion = LocalCompletion::new(&self.cache);
                    lists.push(local_completion.complete(
                        completion_info.pos.clone(),
                        params.text_document_position.text_document.uri.as_str(),
                    ));
                }
            }
            _ => (),
        }

        let is_incomplete = lists.iter().any(|list| list.is_incomplete);
        let completion_list = CompletionList {
            items: lists.into_iter().flat_map(|list| list.items).collect(),
            is_incomplete,
        };
        Ok(CompletionResponse::List(completion_list).into())
    }
}
