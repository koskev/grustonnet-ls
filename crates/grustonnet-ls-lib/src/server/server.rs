use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use lsp_server::{
    Connection, ErrorCode, ExtractError, IoThreads, Message, Notification, Request, RequestId,
    Response, ResponseError,
};
use lsp_types::{
    CompletionList, CompletionOptions, CompletionParams, CompletionResponse,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticParams, DocumentDiagnosticReportResult, InitializeParams,
    PublishDiagnosticsParams, RelatedFullDocumentDiagnosticReport, ServerCapabilities,
    TextDocumentSyncKind, TextDocumentSyncOptions,
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidOpenTextDocument,
        Notification as NotifictionTrait, PublishDiagnostics,
    },
};
use ropey::Rope;
use serde::Serialize;

use crate::{
    cache::Cache,
    completion::{
        Completion, global::GlobalCompletion, keyword::KeywordCompletion, local::LocalCompletion,
    },
    cst::completion::{CompletionInfo, CompletionType},
    diagnostics::{Diagnostics, eval::EvalDiagnostics},
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

macro_rules! lsp_handle_request {
    ($server: expr, $name:ident, $param:ty, $req: expr) => {
        match cast_req::<$param>($req) {
            Ok((_id, params)) => {
                let resp = $server.$name(params);
                return resp;
            }
            Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
            Err(ExtractError::MethodMismatch(req)) => req,
        }
    };
}

macro_rules! lsp_handle_notification {
    ($server: expr, $name:ident, $param:ty, $req: expr) => {
        match cast_notification::<$param>($req) {
            Ok(params) => {
                match $server.$name(params) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Notification failed: {}", e),
                };
                return Ok(());
            }
            Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
            Err(ExtractError::MethodMismatch(req)) => req,
        }
    };
}

#[derive(Default, Debug)]
pub struct LSPResponse(pub serde_json::Value);

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

pub struct LSPServerManager<S: LSPServer> {
    pub server: S,
}

impl<S: LSPServer> LSPServerManager<S> {
    pub fn run(&self) -> Result<()> {
        let server_capabilities = serde_json::to_value(self.server.get_capabilities()).unwrap();
        let params = self
            .server
            .connection()
            .connection
            .initialize(server_capabilities)
            .expect("init connection");

        let _params: InitializeParams = serde_json::from_value(params).unwrap();
        eprintln!("starting example main loop");
        for msg in &self.server.connection().connection.receiver {
            match msg {
                Message::Request(req) => {
                    if self.server.connection().connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    let resp = self.handle_request(req.clone());
                    let result: Result<serde_json::Value, ResponseError> = match resp {
                        Ok(val) => Ok(val.into()),
                        Err(e) => Err(e),
                    };

                    self.server.connection().send(Message::Response(Response {
                        id: req.id,
                        result: result.clone().ok(),
                        error: result.err(),
                    }))?
                }
                Message::Response(resp) => {
                    eprintln!("got response: {resp:?}");
                }
                Message::Notification(not) => {
                    let _ = self.handle_notification(not.clone());
                }
            }
        }
        if let Some(threads) = self.server.connection().threads.lock().unwrap().take() {
            threads.join().unwrap();
        }
        Ok(())
    }
    fn handle_request(&self, req: Request) -> Result<LSPResponse, ResponseError> {
        let _req =
            lsp_handle_request!(self.server, completion, lsp_types::request::Completion, req);

        Err(ResponseError {
            code: ErrorCode::MethodNotFound as i32,
            message: "Method not implemented".into(),
            data: None,
        })
    }

    fn handle_notification(&self, req: Notification) -> Result<(), ResponseError> {
        let mut req = lsp_handle_notification!(
            self.server,
            did_change_configuration,
            DidChangeConfiguration,
            req
        );
        req = lsp_handle_notification!(self.server, did_change_text, DidChangeTextDocument, req);
        req = lsp_handle_notification!(self.server, did_open, DidOpenTextDocument, req);

        let _ = req;
        Err(ResponseError {
            code: ErrorCode::MethodNotFound as i32,
            message: "Method not implemented".into(),
            data: None,
        })
    }
}

pub struct LSPConnection {
    pub connection: Connection,
    pub threads: Arc<Mutex<Option<IoThreads>>>,
}

impl Default for LSPConnection {
    fn default() -> Self {
        let (connection, threads) = Connection::stdio();
        Self {
            connection,
            threads: Arc::new(Mutex::new(Some(threads))),
        }
    }
}

impl LSPConnection {
    pub fn new_network(port: u16) -> Self {
        let (connection, io_threads) = Connection::listen(format!("127.0.0.1:{}", port)).unwrap();
        Self {
            connection: connection,
            threads: Arc::new(Mutex::new(Some(io_threads))),
            ..Default::default()
        }
    }

    pub fn send(&self, message: Message) -> Result<()> {
        Ok(self.connection.sender.send(message)?)
    }
}

fn cast_req<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<R>(not: Notification) -> Result<R::Params, ExtractError<Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    not.extract(R::METHOD)
}

// TODO: Do Generic magic?
#[allow(unused_variables)]
pub trait LSPServer {
    fn connection(&self) -> &LSPConnection;
    fn get_capabilities(&self) -> ServerCapabilities;

    lsp_function_req!(completion, CompletionParams);
    lsp_function_req!(document_diagnostics, DocumentDiagnosticParams);
    lsp_function_req!(publish_diagnostics, &str);

    // Notifications

    lsp_function_not!(did_change_configuration, DidChangeConfigurationParams);
    lsp_function_not!(did_change_text, DidChangeTextDocumentParams);
    lsp_function_not!(did_open, DidOpenTextDocumentParams);
}
