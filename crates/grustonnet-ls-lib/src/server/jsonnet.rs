use std::sync::{Arc, RwLock};

use anyhow::Result;
use language_server::{
    cache::Cache,
    diagnostics::Diagnostics,
    server::{LSPConnection, LSPResponse, LSPServer},
};
use lsp_server::ResponseError;
use lsp_types::{
    CompletionList, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeConfigurationParams, DocumentDiagnosticParams, DocumentDiagnosticReportResult,
    RelatedFullDocumentDiagnosticReport, ServerCapabilities, TextDocumentSyncKind,
    TextDocumentSyncOptions,
};

use crate::{
    cache::JsonnetASTGenerator,
    completion::{
        Completion, global::GlobalCompletion, keyword::KeywordCompletion, local::LocalCompletion,
    },
    cst::completion::{CompletionInfo, CompletionType},
    diagnostics::{eval::EvalDiagnostics, lint::LintDiagnostics},
    node::Node,
    server::config::Configuration,
};

#[derive(Default)]
pub struct JsonnetServer {
    pub cache: Cache<JsonnetASTGenerator, Node>,

    pub connection: LSPConnection,

    pub configuration: Arc<RwLock<Configuration>>,
}

impl JsonnetServer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl LSPServer for JsonnetServer {
    type AstNode = Node;
    type AstGenerator = JsonnetASTGenerator;
    fn connection(&self) -> &LSPConnection {
        &self.connection
    }

    fn cache(&self) -> &Cache<Self::AstGenerator, Self::AstNode> {
        &self.cache
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
        let new_config = Configuration::try_from(params)?;

        // TODO: revisit config architecture
        *self.cache.ast_generator.jsonnet.config.write().unwrap() = new_config.jsonnet.clone();

        *self.configuration.write().unwrap() = new_config;
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
}
