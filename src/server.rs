use anyhow::{Result, anyhow};
use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionOptions, CompletionParams,
    CompletionResponse, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    GotoDefinitionParams, InitializeParams, ServerCapabilities, TextDocumentSyncKind,
    TextDocumentSyncOptions,
};
use serde::Serialize;

use crate::{
    cache::Cache,
    node::{NodeKind, TypedDebug},
};

#[derive(Default)]
pub struct LSPResponse(serde_json::Value);

impl<S: Serialize> From<S> for LSPResponse {
    fn from(value: S) -> Self {
        match serde_json::to_value(value) {
            Ok(val) => LSPResponse(val),
            Err(_) => LSPResponse::default(),
        }
    }
}

impl Into<serde_json::Value> for LSPResponse {
    fn into(self) -> serde_json::Value {
        self.0
    }
}

fn not_implemented_error() -> ResponseError {
    ResponseError {
        code: ErrorCode::MethodNotFound as i32,
        message: "Method not implemented".into(),
        data: None,
    }
}

// TODO: Do Generic magic?
#[allow(unused_variables)]
pub trait LSPServer {
    fn get_capabilities(&self) -> ServerCapabilities;

    fn initialize(&self, params: InitializeParams) -> Result<LSPResponse, ResponseError> {
        Err(not_implemented_error())
    }

    fn goto_definition(&self, params: GotoDefinitionParams) -> Result<LSPResponse, ResponseError> {
        Err(not_implemented_error())
    }

    fn completion(&self, params: CompletionParams) -> Result<LSPResponse, ResponseError> {
        Err(not_implemented_error())
    }

    // Notifications

    fn did_change_configuration(&self, params: DidChangeConfigurationParams) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    fn did_change_text(&self, params: DidChangeTextDocumentParams) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }
}

#[derive(Default, Debug)]
pub struct JsonnetServer {
    cache: Cache,
}

impl JsonnetServer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl LSPServer for JsonnetServer {
    fn get_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
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

    fn did_change_text(&self, params: DidChangeTextDocumentParams) -> Result<()> {
        let mut params = params.clone();
        if let Some(change) = params.content_changes.pop() {
            self.cache
                .update_content(params.text_document.uri.as_str(), &change.text);
        }
        Ok(())
    }

    fn completion(&self, params: CompletionParams) -> Result<LSPResponse, ResponseError> {
        // Global completion
        let doc = self
            .cache
            .get_document(params.text_document_position.text_document.uri.as_str())
            .unwrap();
        //eprintln!("########: {:?}", (*doc.ast.node_kind).typed_debug());

        let stack = doc
            .ast
            .get_stack_by_position(&params.text_document_position.position.into());
        eprintln!("STACK: {:?}", stack.typed_debug());
        for node in &stack.stack {
            eprintln!("Node of Type {}", (*node.node_kind).variant_name(),)
        }
        let items: Vec<CompletionItem> = stack
            .stack
            .iter()
            .filter_map(|node| match &(*node.node_kind) {
                crate::node::NodeKind::LocalBind(bind) => {
                    eprintln!("Got bind!");
                    Some(CompletionItem {
                        label: bind.variable.clone(),
                        ..Default::default()
                    })
                }
                NodeKind::Local { binds, body } => {
                    eprintln!("Got local!");

                    Some(CompletionItem {
                        label: binds[0].variable.clone(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        ..Default::default()
                    })
                }
                _ => {
                    eprintln!("No bind {}", node.node_kind.variant_name());
                    None
                }
            })
            .collect();
        eprintln!("ITEMS: {:?}", items);
        match *doc.ast.node_kind {
            crate::node::NodeKind::LocalBind(_) => eprintln!("LocalBind"),
            _ => eprintln!("Unkown root node"),
        }
        Ok(CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items,
        })
        .into())
    }
}
