use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use bevy_tasks::TaskPool;
use language_server::{
    cache::Cache,
    completion::Completion,
    diagnostics::Diagnostics,
    server::{LSPConnection, LSPError, LSPResponse, LSPServer, get_response_error},
    utils::diff,
};
use lsp_types::{
    CompletionList, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeConfigurationParams, DocumentDiagnosticParams, DocumentDiagnosticReportResult,
    GotoDefinitionParams, GotoDefinitionResponse, InitializeParams, InlayHint, InlayHintParams,
    Location, OneOf, Range, RelatedFullDocumentDiagnosticReport, ServerCapabilities,
    TextDocumentSyncKind, TextDocumentSyncOptions, Uri, WorkspaceFolder,
};

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    completion::{
        global::GlobalCompletion,
        keyword::KeywordCompletion,
        local::{CallStackIter, LocalCompletion, ResolveNodeIter},
    },
    cst::completion::{CompletionInfo, CompletionType},
    diagnostics::{eval::EvalDiagnostics, lint::LintDiagnostics},
    inlay_hint::{Inlay, debug::DebugInlay},
    node::{
        DesugaredObject, DesugaredObjectField, LiteralString, Node, NodeKind,
        location::LocationRange,
    },
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

    fn handle_init_parameters(&self, params: InitializeParams) {
        let workspaces = params.workspace_folders.unwrap_or_default();

        if workspaces.len() > 0 {
            self.cache
                .ast_generator
                .jsonnet
                .set_root_dir(workspaces.first().unwrap().uri.path().as_str());
        }
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
            definition_provider: Some(OneOf::Left(true)),
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
        self.cache
            .ast_generator
            .jsonnet
            .set_config(&new_config.jsonnet);

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

        let config = self.configuration.read().unwrap().clone();
        let mut completion_list: Vec<Box<dyn Completion>> = vec![];
        match completion_info.completion_type {
            CompletionType::Global => {
                // Global completion
                if config.completion.enable_global {
                    let global_completion = GlobalCompletion::new(&self.cache);
                    completion_list.push(Box::new(global_completion));
                }
                // Keyword completion
                if config.completion.enable_keywords {
                    let keyword_completion = KeywordCompletion::new(&self.cache);
                    completion_list.push(Box::new(keyword_completion));
                }
            }
            CompletionType::Local => {
                if config.completion.enable_local {
                    let local_completion = LocalCompletion::new(&self.cache);
                    completion_list.push(Box::new(local_completion));
                }
            }
            _ => (),
        }

        let pool = TaskPool::new();
        let filename = params.text_document_position.text_document.uri.as_str();
        let lists = pool.scope(|s| {
            for provider in completion_list {
                let location = completion_info.pos.clone().into();
                s.spawn(async move { provider.complete(location, filename) });
            }
        });

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

    fn goto_definition(&self, params: GotoDefinitionParams) -> Result<LSPResponse, LSPError> {
        // Get selected node
        let doc = self.cache.get_document(
            params
                .text_document_position_params
                .text_document
                .uri
                .as_str(),
        )?;

        let pos = params.text_document_position_params.position;

        let stack = doc.get_ast()?.get_stack_by_position(&(pos.into()));

        let mut document_stack = stack;
        let mut call_stack = document_stack
            .peek()
            .ok_or(anyhow!("document stack is empty"))?
            .get_call_stack();
        let mut index_name = String::new();
        let built_node = match call_stack.stack.len() {
            x if x == 1 => call_stack.stack.pop().expect("impossible to reach"),
            x if x > 1 => {
                // Remove the last node (=at the beginning of the vec) and resolve the rest of the stack
                let last_node = call_stack.stack.remove(0);
                index_name = match last_node.node_kind.as_ref() {
                    NodeKind::Index(idx) => {
                        idx.get_name().ok_or(anyhow!("could not get index name"))?
                    }
                    NodeKind::Apply(func) => {
                        func.get_name().ok_or(anyhow!("could not get apply name"))?
                    }
                    _ => "".to_string(),
                };
                let call_iter = CallStackIter::new_with_call_stack(
                    &self.cache,
                    &mut document_stack,
                    call_stack,
                )
                .ok_or(anyhow!("could not resolve call stack"))?;
                call_iter
                    .last()
                    .ok_or(anyhow!("Call iter was empty. Can't goto definition"))?
            }
            _ => {
                return Err(anyhow!("Cant find the destination of an empty stack").into());
            }
        };

        let location: LocationRange = match built_node.node_kind.as_ref() {
            NodeKind::Var(var) => Some(
                var.resolve_bind(&document_stack)
                    .ok_or(anyhow!("unable to resolve var"))?
                    .loc_range
                    .clone(),
            ),
            NodeKind::DesugaredObject(obj) => Some(
                obj.get_field(&index_name)
                    .ok_or(anyhow!("unable to get object field"))?
                    .loc_range
                    .clone(),
            ),
            _ => None,
        }
        .ok_or(anyhow!(
            "Could not resolve location of {}",
            built_node.node_kind
        ))?;

        Ok(GotoDefinitionResponse::Scalar(Location {
            uri: Uri::from_str(&built_node.node_base.loc_range.file_name)
                .map_err(|e| anyhow!("Parsing uri from node {}", e))?,
            range: Range {
                start: location.begin.into(),
                end: location.end.into(),
            },
        })
        .into())
    }

    fn inlay_hint(&self, params: InlayHintParams) -> Result<LSPResponse, LSPError> {
        let mut hints: Vec<InlayHint> = vec![];

        if self.configuration.read().unwrap().inlay.enable_debug {
            let debug_hints =
                DebugInlay::new(&self.cache).inlay(params.text_document.uri.as_str())?;
            hints.extend(debug_hints);
        }

        Ok(hints.into())
    }
}
