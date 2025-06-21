use anyhow::{Result, anyhow};
use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionOptions, CompletionParams,
    CompletionResponse, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, InitializeParams, ServerCapabilities,
    TextDocumentSyncKind, TextDocumentSyncOptions, notification::DidOpenTextDocument,
};
use ropey::Rope;
use serde::Serialize;

use crate::{
    cache::Cache,
    completion::{Completion, global::GlobalCompletion},
    node::{NodeKind, TypedDebug},
};

macro_rules! lsp_function_req {
    ($name:ident, $param:ty) => {
        fn $name(&self, params: $param) -> Result<LSPResponse, ResponseError> {
            Err(not_implemented_error())
        }
    };
}

macro_rules! lsp_function_not {
    ($name:ident, $param:ty) => {
        fn $name(&self, params: $param) -> Result<()> {
            Err(anyhow!("Not implemented"))
        }
    };
}

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

    lsp_function_req!(completion, CompletionParams);

    // Notifications

    lsp_function_not!(did_change_configuration, DidChangeConfigurationParams);
    lsp_function_not!(did_change_text, DidChangeTextDocumentParams);
    lsp_function_not!(did_open, DidOpenTextDocumentParams);
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
        }
        Ok(())
    }

    fn did_open(&self, params: DidOpenTextDocumentParams) -> Result<()> {
        self.cache.update_content(
            params.text_document.uri.as_str(),
            &params.text_document.text,
        );

        Ok(())
    }

    fn completion(&self, params: CompletionParams) -> Result<LSPResponse, ResponseError> {
        // Global completion
        let global_completion = GlobalCompletion::new(&self.cache);
        let mut lists = vec![];
        lists.push(global_completion.complete(
            params.text_document_position.position.into(),
            params.text_document_position.text_document.uri.as_str(),
        ));

        let is_incomplete = lists.iter().any(|list| list.is_incomplete);
        let completion_list = CompletionList {
            items: lists.into_iter().flat_map(|list| list.items).collect(),
            is_incomplete,
        };
        Ok(CompletionResponse::List(completion_list).into())
    }
}
