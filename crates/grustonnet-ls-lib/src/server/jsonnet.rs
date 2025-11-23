use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::Result;
use bevy_tasks::TaskPool;
use grustonnet_config::{Configuration, VariableNaming};
use jsonnet_cst::{
    completion::{CompletionInfo, CompletionType},
    node::JsonnetNode,
};
use jsonnet_location::{Location, LocationRange};
use language_server::{
    cache::Cache,
    completion::Completion,
    diagnostics::{Diagnostics, DiagnosticsQueue, DiagnosticsResult},
    server::{
        LSPConnection, LSPError, LSPResponse, LSPServer, WorkProgressSender, get_response_error,
    },
    utils::diff,
};
use log::error;
use lsp_types::{
    CodeActionOrCommand, CodeActionProviderCapability, CompletionList, CompletionOptions,
    CompletionParams, CompletionResponse, DidChangeConfigurationParams, DocumentDiagnosticParams,
    DocumentDiagnosticReportResult, ExecuteCommandOptions, GotoDefinitionParams,
    GotoDefinitionResponse, InitializeParams, InlayHint, InlayHintParams, OneOf,
    ParameterInformation, ParameterLabel, RelatedFullDocumentDiagnosticReport, SemanticTokens,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelp,
    SignatureHelpOptions, SignatureInformation, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
};

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    command::handle_command,
    completion::{
        global::GlobalCompletion, import::ImportCompletion, keyword::KeywordCompletion,
        local::LocalCompletion, snippets::docsonnet::DocsonnetSnippets,
    },
    definition::DefinitionProvider,
    diagnostics::{
        ASTDiagnosticsHandler, JsonnetDiagnostics,
        cst_linters::local_function::LocalFunctionDiagnostics,
        eval::EvalDiagnostics,
        filter::JsonnetDiagnosticFilter,
        go_lint::GoLintDiagnostics,
        linters::{
            self,
            dollar::DollarDiagnostics,
            duplicate_values::DuplicateValuesDiagnostic,
            recursive_argument::RecursiveArgumentDiagnostic,
            shadow_variable::ShadowVariableDiagnostics,
            variable_naming::{SnakeCaseDiagnostics, VariableNamingDiagnostics},
        },
    },
    documentation::DocumentationInfo,
    inlay_hint::{Inlay, apply::ApplyInlay, debug::DebugInlay, name::NameInlay},
    node::{NodeHelper, Stackhelper},
    references::ReferenceProvider,
    rename::RenameProvider,
    semantic_tokens::{self, SemanticDataList},
    utils,
};

#[derive(Default)]
pub struct JsonnetServer {
    pub cache: Cache<JsonnetASTGenerator>,

    pub connection: LSPConnection,

    pub configuration: Arc<RwLock<Configuration>>,

    pub diagnostics_queue: Option<DiagnosticsQueue<JsonnetDiagnosticFilter>>,
}

impl JsonnetServer {
    pub fn new(connection: LSPConnection) -> Self {
        let cache = Cache::default();
        let diagnostics_queue = DiagnosticsQueue::new(
            connection.connection.sender.clone(),
            JsonnetDiagnosticFilter::new(cache.clone()),
        );
        let task_queue = diagnostics_queue.clone();
        bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default)
            .spawn(async move {
                task_queue.run();
            })
            .detach();
        Self {
            diagnostics_queue: Some(diagnostics_queue),
            connection,
            cache,
            ..Default::default()
        }
    }

    pub fn get_diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult> {
        let diags = self.get_diagnostics_provider();
        diags
            .iter()
            .flat_map(|diag| diag.diagnostics(uri))
            .collect()
    }

    fn get_diagnostics_provider(&self) -> Vec<Box<dyn Diagnostics>> {
        let config = self.configuration.read().unwrap().clone();
        let mut diags: Vec<Box<dyn Diagnostics>> = vec![];
        if config.diagnostics.enable_eval {
            diags.push(Box::new(EvalDiagnostics::new(self.cache.clone())));
        }
        if config.diagnostics.enable_go_lint {
            diags.push(Box::new(GoLintDiagnostics::new(self.cache.clone())));
        }
        if config.diagnostics.unused_variables {
            diags.push(Box::new(linters::unused::UnusedDiagnostics::new(
                self.cache.clone(),
            )));
        }

        // TODO: Add a macro for all those settings
        let mut diagnostics_handler_diags: Vec<Box<dyn JsonnetDiagnostics>> = vec![];

        macro_rules! add_jsonnet_diag {
            ($config_name: ident, $diag: ty) => {
                if config.diagnostics.$config_name {
                    diagnostics_handler_diags.push(Box::new(<$diag>::default()));
                }
            };
        }

        if let Some(naming_diag) = match config.diagnostics.variable_naming {
            VariableNaming::SnakeCase => Some(Box::new(VariableNamingDiagnostics::<
                SnakeCaseDiagnostics,
            >::new())),
            VariableNaming::None => None,
        } {
            diagnostics_handler_diags.push(naming_diag);
        }

        add_jsonnet_diag!(prevent_dollar, DollarDiagnostics);
        add_jsonnet_diag!(recursive_arguments, RecursiveArgumentDiagnostic);
        add_jsonnet_diag!(shadow_variable, ShadowVariableDiagnostics);

        diagnostics_handler_diags.push(Box::new(DuplicateValuesDiagnostic {
            config: config.diagnostics.duplicate_detection.clone(),
            ..Default::default()
        }));

        if config.diagnostics.local_function {
            diags.push(Box::new(LocalFunctionDiagnostics {
                cache: self.cache.clone(),
            }));
        }

        diags.push(Box::new(ASTDiagnosticsHandler {
            cache: self.cache.clone(),
            diags: diagnostics_handler_diags,
        }));
        diags
    }
}

impl LSPServer for JsonnetServer {
    type AstGenerator = JsonnetASTGenerator;
    fn connection(&self) -> &LSPConnection {
        &self.connection
    }

    fn queue_diagnostics(&self, uri: &Uri) {
        let diags = self.get_diagnostics_provider();
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
                commands: vec![
                    "jsonnet.evalFile".into(),
                    "config.jpaths".into(),
                    "config.extcode".into(),
                    "config.extvars".into(),
                ],
                ..Default::default()
            }),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".into(), ",".into()]),
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
                        items: self
                            .get_diagnostics(&params.text_document.uri)
                            .into_iter()
                            .map(|d| d.diagnostics)
                            .collect(),
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

                if config.completion.snippets.docsonnet {
                    completion_list.push(Box::new(DocsonnetSnippets {}));
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

    fn code_action(
        &self,
        params: <lsp_types::request::CodeActionRequest as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let actions: Vec<CodeActionOrCommand> = self
            .diagnostics_queue
            .clone()
            .unwrap()
            .current_diagnostics
            .read()
            .unwrap()
            .iter()
            .flat_map(|(_, d)| {
                d.iter().flat_map(|d| {
                    d.1.iter()
                        .filter(|d| {
                            let locrange: LocationRange = LocationRange {
                                begin: d.diagnostics.range.start.into(),
                                end: d.diagnostics.range.end.into(),
                                ..Default::default()
                            };
                            locrange.in_range(&params.range.start.into())
                        })
                        .flat_map(|d| {
                            d.code_actions
                                .iter()
                                .map(|action| CodeActionOrCommand::CodeAction(action.clone()))
                        })
                })
            })
            .collect();
        Ok(actions.into())
    }

    fn signature_help(
        &self,
        params: <lsp_types::request::SignatureHelpRequest as lsp_types::request::Request>::Params,
    ) -> Result<LSPResponse, LSPError> {
        let doc = self
            .cache
            .get_document(&params.text_document_position_params.text_document.uri)?;
        let ast = doc.get_ast()?;

        let stack =
            ast.get_stack_by_position(&params.text_document_position_params.position.into());

        Ok(stack
            .stack
            .iter()
            .find_map(|n| {
                let apply_function_data = n.get_apply_function(ast.clone(), &self.cache)?;
                let func_name = apply_function_data
                    .apply
                    .get_name()
                    .unwrap_or("unknown".into());
                let func_params = &apply_function_data.function.parameters;
                let names: Vec<String> = func_params.iter().map(|p| p.name.0.clone()).collect();
                let cst_tree = jsonnet_cst::new_tree(&doc.content)?;
                let cst_loc: Location = params.text_document_position_params.position.into();
                let root_node = cst_tree.root_node();
                let cst_node = root_node.get_node_at(cst_loc.into())?;
                let active_param = cst_node.get_param_pos();
                Some(SignatureHelp {
                    signatures: vec![SignatureInformation {
                        label: format!("{}({})", func_name, names.join(", ")),
                        active_parameter: Some(active_param),
                        documentation: None,
                        parameters: Some(
                            names
                                .iter()
                                .map(|name| ParameterInformation {
                                    label: ParameterLabel::Simple(name.clone()),
                                    // TODO: get docsonnet documentation
                                    documentation: None,
                                })
                                .collect(),
                        ),
                    }],
                    active_signature: Some(0),
                    active_parameter: Some(active_param),
                })
            })
            .into())
    }
}
