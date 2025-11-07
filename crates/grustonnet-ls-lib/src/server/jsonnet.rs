use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::Result;
use bevy_tasks::TaskPool;
use language_server::{
    cache::Cache,
    completion::Completion,
    diagnostics::{Diagnostics, DiagnosticsQueue},
    server::{
        LSPConnection, LSPError, LSPResponse, LSPServer, WorkProgressSender, get_response_error,
    },
    utils::diff,
};
use lsp_types::{
    CompletionList, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeConfigurationParams, DocumentDiagnosticParams, DocumentDiagnosticReportResult,
    ExecuteCommandOptions, GotoDefinitionParams, GotoDefinitionResponse, InitializeParams,
    InlayHint, InlayHintParams, OneOf, RelatedFullDocumentDiagnosticReport, SemanticTokens,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities,
    TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
};

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    command::handle_command,
    completion::{
        global::GlobalCompletion, import::ImportCompletion, keyword::KeywordCompletion,
        local::LocalCompletion,
    },
    cst::completion::{CompletionInfo, CompletionType},
    definition::DefinitionProvider,
    diagnostics::{eval::EvalDiagnostics, go_lint::GoLintDiagnostics, lint::LintDiagnostics},
    inlay_hint::{Inlay, apply::ApplyInlay, debug::DebugInlay, name::NameInlay},
    references::ReferenceProvider,
    rename::RenameProvider,
    semantic_tokens::{self, SemanticDataList},
    server::config::Configuration,
    utils,
};

#[derive(Default)]
pub struct JsonnetServer {
    pub cache: Cache<JsonnetASTGenerator>,

    pub connection: LSPConnection,

    pub configuration: Arc<RwLock<Configuration>>,

    pub diagnostics_queue: Option<DiagnosticsQueue>,
}

impl JsonnetServer {
    pub fn new(connection: LSPConnection) -> Self {
        let diagnostics_queue = DiagnosticsQueue::new(connection.connection.sender.clone());
        let task_queue = diagnostics_queue.clone();
        bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default)
            .spawn(async move {
                task_queue.run();
            })
            .detach();
        Self {
            diagnostics_queue: Some(diagnostics_queue),
            connection,
            ..Default::default()
        }
    }

    pub fn get_diagnostics(&self, uri: &Uri) -> Vec<Diagnostic> {
        let mut items = vec![];
        let config = self.configuration.read().unwrap().clone();
        if config.diagnostics.enable_eval {
            let diags = EvalDiagnostics::new(self.cache.clone()).diagnostics(uri);
            items.extend(diags);
        }
        if config.diagnostics.enable_go_lint {
            let diags = GoLintDiagnostics::new(self.cache.clone()).diagnostics(uri);
            items.extend(diags);
        }
        if config.diagnostics.enable_lint {
            let diags = LintDiagnostics::new(self.cache.clone()).diagnostics(uri);
            items.extend(diags);
        }
        // TODO: Filter messages with the same target but different severity
        items
    }
}

impl LSPServer for JsonnetServer {
    type AstGenerator = JsonnetASTGenerator;
    fn connection(&self) -> &LSPConnection {
        &self.connection
    }

    fn queue_diagnostics(&self, uri: &Uri) {
        let config = self.configuration.read().unwrap().clone();
        let mut diags: Vec<Box<dyn Diagnostics>> = vec![];
        if config.diagnostics.enable_eval {
            diags.push(Box::new(EvalDiagnostics::new(self.cache.clone())));
        }
        if config.diagnostics.enable_go_lint {
            diags.push(Box::new(GoLintDiagnostics::new(self.cache.clone())));
        }
        if config.diagnostics.enable_lint {
            diags.push(Box::new(LintDiagnostics::new(self.cache.clone())));
        }
        if let Some(queue) = self.diagnostics_queue.as_ref() {
            queue.queue(uri.clone(), diags);
        }
    }

    fn handle_init_parameters(&self, params: InitializeParams) {
        let workspaces = params.workspace_folders.unwrap_or_default();

        if !workspaces.is_empty() {
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
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    full: Some(lsp_types::SemanticTokensFullOptions::Bool(true)),
                    range: Some(false),
                    legend: semantic_tokens::get_token_map(),
                    ..Default::default()
                }),
            ),
            references_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Left(true)),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec!["jsonnet.evalFile".into()],
                ..Default::default()
            }),
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

        // TODO: revisit config architecture. this is so cursed
        self.cache
            .ast_generator
            .jsonnet
            .set_config(&new_config.jsonnet);

        *self.configuration.write().unwrap() = new_config.clone();

        if new_config.jsonnet.preload_files {
            let eval_params = self.cache.ast_generator.jsonnet.get_evaluate_params(".");
            let all_files = utils::files::get_all_jsonnnet_files(&eval_params.jpaths);
            let cache = self.cache.clone();
            let sender = self.connection.connection.sender.clone();
            bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default)
                .spawn(async move {
                    let mut progress = WorkProgressSender::new(sender);
                    progress.work_start("Analyzing workspace".into(), Some("Test".into()));
                    for (i, uri) in all_files.iter().enumerate() {
                        let _ = cache.get_document(uri);
                        progress.work_progress(
                            (i * 100 / all_files.len()) as u32,
                            Some(format!("Loading file {}/{}", i, all_files.len())),
                        );
                    }
                    progress.work_done();
                })
                .detach();
        }

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
                        items: self.get_diagnostics(&params.text_document.uri),
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
            .get_document(&params.text_document_position.text_document.uri)?;

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
            CompletionType::Import => {
                log::info!("Import completion");
                let import_completion = ImportCompletion::new(&self.cache);
                completion_list.push(Box::new(import_completion));
            }
            _ => (),
        }

        let pool = TaskPool::new();
        let lists = pool.scope(|s| {
            for provider in completion_list {
                let location = completion_info.pos.clone().into();
                let uri = params.text_document_position.text_document.uri.clone();
                s.spawn(async move { provider.complete(location, &uri) });
            }
        });

        let failed: Vec<_> = lists.iter().filter_map(|res| res.as_ref().err()).collect();
        let succeeded: Vec<&CompletionList> =
            lists.iter().filter_map(|res| res.as_ref().ok()).collect();

        if succeeded.is_empty() && !failed.is_empty() {
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
                .filter(|item| {
                    !config.completion.hide_docsonnet_members || !item.label.starts_with("#")
                })
                .collect(),
            is_incomplete,
        };

        Ok(CompletionResponse::List(completion_list).into())
    }

    fn formatting(
        &self,
        params: <lsp_types::request::Formatting as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let uri = params.text_document.uri;
        let options = &self.configuration.read().unwrap().format;
        let doc = self.cache.get_document(&uri)?;
        let formatted = match self.cache.ast_generator.jsonnet.format_snippet(
            uri.as_str(),
            &doc.content,
            options,
        ) {
            Ok(res) => res,
            Err(e) => return Err(e.into()),
        };

        let edits = diff::get_text_edits(&doc.content, &formatted);

        Ok(edits.into())
    }

    fn goto_definition(&self, params: GotoDefinitionParams) -> Result<LSPResponse, LSPError> {
        let pos = params.text_document_position_params.position;

        let info = DefinitionProvider::new(&self.cache).definition(
            &params.text_document_position_params.text_document.uri,
            pos.into(),
        )?;

        Ok(GotoDefinitionResponse::Scalar(info.location).into())
    }

    fn inlay_hint(&self, params: InlayHintParams) -> Result<LSPResponse, LSPError> {
        let mut hints: Vec<InlayHint> = vec![];
        let config = self.configuration.read().unwrap();

        if config.inlay.enable_debug {
            let debug_hints =
                DebugInlay::new(&self.cache).inlay(&params.text_document.uri, params.range)?;
            hints.extend(debug_hints);
        }

        if config.inlay.name_hints.enabled {
            let function_end_hints =
                NameInlay::new(&self.cache, config.inlay.name_hints.line_threshold)
                    .inlay(&params.text_document.uri, params.range)?;
            hints.extend(function_end_hints);
        }

        if config.inlay.enable_function_parameters {
            let argument_hints =
                ApplyInlay::new(&self.cache).inlay(&params.text_document.uri, params.range)?;
            hints.extend(argument_hints);
        }

        Ok(hints.into())
    }

    fn semantic_tokens(
        &self,
        params: <lsp_types::request::SemanticTokensFullRequest as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let config = self.configuration.read().unwrap();
        let start = Instant::now();
        let doc = self.cache.get_document(&params.text_document.uri)?;
        let root = doc.get_ast()?;
        let mut tokens = SemanticDataList::default();
        if config.semantic_tokens.semantic_tokens {
            tokens.data.extend(semantic_tokens::get_tokens(root).data);
        }
        if config.semantic_tokens.treesitter_tokens {
            tokens
                .data
                .extend(semantic_tokens::treesitter_bridge::get_tokens(doc).data);
        }

        log::info!("Getting semantic tokens took {:?}", start.elapsed());
        let semantic_tokens: SemanticTokens = tokens.into();
        Ok(semantic_tokens.into())
    }

    fn references(
        &self,
        params: <lsp_types::request::References as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let start = Instant::now();
        let mut search_paths = self
            .cache
            .ast_generator
            .jsonnet
            .params
            .read()
            .unwrap()
            .jpaths
            .clone();
        search_paths.push(
            self.cache
                .ast_generator
                .jsonnet
                .root_dir
                .read()
                .unwrap()
                .clone(),
        );
        let references = ReferenceProvider::new(&self.cache, &search_paths).references(
            params.text_document_position.position.into(),
            &params.text_document_position.text_document.uri,
            params.context.include_declaration,
        )?;
        log::info!("Finding references took {:?}", start.elapsed());

        Ok(references.into())
    }

    fn rename(
        &self,
        params: <lsp_types::request::Rename as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let mut search_paths = self
            .cache
            .ast_generator
            .jsonnet
            .params
            .read()
            .unwrap()
            .jpaths
            .clone();
        search_paths.push(
            self.cache
                .ast_generator
                .jsonnet
                .root_dir
                .read()
                .unwrap()
                .clone(),
        );

        Ok(RenameProvider::new(&self.cache)
            .rename(params, &search_paths)?
            .into())
    }

    fn execute_command(
        &self,
        params: <lsp_types::request::ExecuteCommand as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        handle_command(&self.cache, params)
    }
}
