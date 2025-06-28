use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use language_server::{
    cache::Cache,
    diagnostics::Diagnostics,
    server::{LSPConnection, LSPError, LSPResponse, LSPServer, get_response_error},
    utils::diff,
};
use lsp_types::{
    CompletionList, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeConfigurationParams, DocumentDiagnosticParams, DocumentDiagnosticReportResult,
    GotoDefinitionParams, GotoDefinitionResponse, InlayHint, InlayHintParams, OneOf, Range,
    RelatedFullDocumentDiagnosticReport, ServerCapabilities, TextDocumentSyncKind,
    TextDocumentSyncOptions, Uri,
};

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    completion::{
        Completion, global::GlobalCompletion, keyword::KeywordCompletion, local::LocalCompletion,
    },
    cst::completion::{CompletionInfo, CompletionType},
    diagnostics::{eval::EvalDiagnostics, lint::LintDiagnostics},
    node::{DesugaredObject, DesugaredObjectField, LiteralString, LocalBind, Node, NodeKind},
    server::config::Configuration,
};

#[derive(Default)]
pub struct JsonnetServer {
    pub cache: Cache<JsonnetASTGenerator>,

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
    type AstGenerator = JsonnetASTGenerator;
    fn connection(&self) -> &LSPConnection {
        &self.connection
    }

    fn cache(&self) -> &Cache<Self::AstGenerator> {
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
            document_formatting_provider: Some(OneOf::Left(true)),
            inlay_hint_provider: Some(OneOf::Left(true)),
            ..Default::default()
        }
    }

    fn did_change_configuration(
        &self,
        params: DidChangeConfigurationParams,
    ) -> Result<(), LSPError> {
        let new_config = match Configuration::try_from(params) {
            Ok(conf) => conf,
            Err(e) => {
                return Err(get_response_error(format!(
                    "Could not parse the configuration: {}",
                    e
                )));
            }
        };

        // TODO: revisit config architecture
        *self.cache.ast_generator.jsonnet.config.write().unwrap() = new_config.jsonnet.clone();

        *self.configuration.write().unwrap() = new_config;
        Ok(())
    }

    fn document_diagnostics(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<LSPResponse, LSPError> {
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

    fn completion(&self, params: CompletionParams) -> Result<LSPResponse, LSPError> {
        let doc = self
            .cache
            .get_document(params.text_document_position.text_document.uri.as_str())?;

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
        let failed: Vec<_> = lists.iter().filter_map(|res| res.as_ref().err()).collect();
        let succeeded: Vec<&CompletionList> =
            lists.iter().filter_map(|res| res.as_ref().ok()).collect();

        if succeeded.len() == 0 && failed.len() > 0 {
            let first_err = *failed.first().unwrap();
            return Err(first_err.into());
        }

        for err in failed {
            log::error!("Failed to complete: {}", err)
        }

        let is_incomplete = succeeded.iter().any(|list| list.is_incomplete);
        let completion_list = CompletionList {
            items: succeeded
                .into_iter()
                .flat_map(|list| list.items.clone())
                .collect(),
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

    fn formatting(
        &self,
        params: <lsp_types::request::Formatting as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let filename = params.text_document.uri.as_str();
        let options = &self.configuration.read().unwrap().format;
        let doc = self.cache.get_document(filename)?;
        let formatted =
            match self
                .cache
                .ast_generator
                .jsonnet
                .format_snippet(filename, &doc.content, &options)
            {
                Ok(res) => res,
                Err(e) => return Err(e.into()),
            };

        let edits = diff::get_text_edits(&doc.content, &formatted);

        Ok(edits.into())
    }
    fn inlay_hint(&self, params: InlayHintParams) -> Result<LSPResponse, LSPError> {
        let doc = self.cache.get_document(params.text_document.uri.as_str())?;

        let doc_stack = doc.get_ast()?.get_complete_stack();
        let hints: Vec<InlayHint> = doc_stack
            .stack
            .iter()
            .map(|n| InlayHint {
                position: n.node_base.loc_range.begin.clone().into(),
                padding_right: Some(true),
                label: lsp_types::InlayHintLabel::String(n.node_kind.variant_name().to_string()),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: None,
                data: None,
            })
            .collect();
        Ok(hints.into())
    }
}
