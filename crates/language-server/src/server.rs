use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::Result;
use crossbeam::channel::Sender;
use lsp_server::{
    Connection, ErrorCode, ExtractError, IoThreads, Message, Notification, Request, RequestId,
    Response, ResponseError,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, ProgressParams, ProgressParamsValue, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressEnd, WorkDoneProgressReport,
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification as NotificationTrait, Progress,
    },
    request::{
        Completion, DocumentDiagnosticRequest, ExecuteCommand, Formatting, GotoDefinition,
        InlayHintRequest, References, Rename, Request as RequestTrait, SemanticTokensFullRequest,
    },
};
use rand::RngCore;
use ropey::Rope;
use serde::Serialize;

use crate::cache::{ASTGenerator, Cache};

macro_rules! lsp_function_req {
    ($name:ident, $req:ty) => {
        fn $name(&self, params: <$req as RequestTrait>::Params) -> Result<LSPResponse, LSPError> {
            Err(not_implemented_error())
        }
    };
}

macro_rules! lsp_function_not {
    ($name:ident, $param:ty) => {
        fn $name(&self, params: <$param as NotificationTrait>::Params) -> Result<(), LSPError> {
            Err(not_implemented_error())
        }
    };
}

macro_rules! lsp_handle_request {
    ($server: expr, $name:ident, $param:ty, $req: expr) => {
        match cast_req::<$param>($req) {
            Ok((_id, params)) => {
                let start = Instant::now();
                let resp = $server.$name(params);
                log::info!("Request {} took {:?}", stringify!($name), start.elapsed());
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
                let start = Instant::now();
                match $server.$name(params) {
                    Ok(_) => (),
                    Err(e) => log::error!("Notification failed: {:?}", e),
                };
                log::info!(
                    "Notification {} took {:?}",
                    stringify!($name),
                    start.elapsed()
                );
                return Ok(());
            }
            Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
            Err(ExtractError::MethodMismatch(req)) => req,
        }
    };
}

#[derive(Default, Debug)]
pub struct LSPError {
    pub message: String,
    pub error_code: i32,
}

impl Error for LSPError {}
impl Display for LSPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO: fix error handling
impl From<ResponseError> for LSPError {
    fn from(value: ResponseError) -> Self {
        Self {
            message: value.message,
            error_code: value.code,
        }
    }
}

impl From<LSPError> for ResponseError {
    fn from(val: LSPError) -> Self {
        ResponseError {
            code: val.error_code,
            message: val.message,
            data: None,
        }
    }
}

impl From<anyhow::Error> for LSPError {
    fn from(value: anyhow::Error) -> Self {
        Self::from(&value)
    }
}

impl From<&anyhow::Error> for LSPError {
    fn from(value: &anyhow::Error) -> Self {
        Self {
            error_code: ErrorCode::UnknownErrorCode as i32,
            message: value.to_string(),
        }
    }
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

impl From<LSPResponse> for serde_json::Value {
    fn from(val: LSPResponse) -> Self {
        val.0
    }
}

fn not_implemented_error() -> LSPError {
    LSPError {
        error_code: ErrorCode::MethodNotFound as i32,
        message: "Method not implemented".into(),
    }
}

pub fn get_response_error(message: String) -> LSPError {
    LSPError {
        error_code: ErrorCode::UnknownErrorCode as i32,
        message,
    }
}

pub struct LSPServerManager<S>
where
    S: LSPServer,
{
    pub server: S,
}

impl<S> LSPServerManager<S>
where
    S: LSPServer,
{
    pub fn run(&self) -> Result<()> {
        let server_capabilities = serde_json::to_value(self.server.get_capabilities()).unwrap();
        let params = self
            .server
            .connection()
            .connection
            .initialize(server_capabilities)
            .expect("init connection");

        let params: InitializeParams = serde_json::from_value(params).unwrap();
        self.server.handle_init_parameters(params);
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
                        Err(e) => Err(e.into()),
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
    fn handle_request(&self, req: Request) -> Result<LSPResponse, LSPError> {
        let mut req =
            lsp_handle_request!(self.server, completion, lsp_types::request::Completion, req);
        req = lsp_handle_request!(self.server, formatting, lsp_types::request::Formatting, req);
        req = lsp_handle_request!(self.server, goto_definition, GotoDefinition, req);
        req = lsp_handle_request!(self.server, inlay_hint, InlayHintRequest, req);
        req = lsp_handle_request!(self.server, references, References, req);
        req = lsp_handle_request!(self.server, rename, Rename, req);
        req = lsp_handle_request!(self.server, semantic_tokens, SemanticTokensFullRequest, req);
        req = lsp_handle_request!(self.server, execute_command, ExecuteCommand, req);

        Err(LSPError {
            error_code: ErrorCode::MethodNotFound as i32,
            message: format!("Method {} not implemented", req.method),
        })
    }

    fn handle_notification(&self, req: Notification) -> Result<(), LSPError> {
        let mut req = lsp_handle_notification!(
            self.server,
            did_change_configuration,
            DidChangeConfiguration,
            req
        );
        req = lsp_handle_notification!(self.server, did_change_text, DidChangeTextDocument, req);
        req = lsp_handle_notification!(self.server, did_open, DidOpenTextDocument, req);
        req = lsp_handle_notification!(self.server, did_close, DidCloseTextDocument, req);

        let _ = req;
        Err(LSPError {
            error_code: ErrorCode::MethodNotFound as i32,
            message: "Method not implemented".into(),
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
            connection,
            threads: Arc::new(Mutex::new(Some(io_threads))),
            ..Default::default()
        }
    }

    pub fn send(&self, message: Message) -> Result<()> {
        Ok(self.connection.sender.send(message)?)
    }
}

#[derive(Debug, Clone)]
pub struct WorkProgressSender {
    sender: Sender<Message>,
    id: u32,
    progress: u32,
    last_message: Option<String>,
}

impl WorkProgressSender {
    pub fn new(sender: Sender<Message>) -> Self {
        let id = rand::rng().next_u32();
        Self {
            sender,
            id,
            progress: 0,
            last_message: None,
        }
    }

    pub fn work_start(&self, title: String, message: Option<String>) {
        self.sender
            .send(Message::Notification(Notification {
                method: Progress::METHOD.to_string(),
                params: serde_json::to_value(ProgressParams {
                    token: lsp_types::NumberOrString::Number(self.id as i32),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                        WorkDoneProgressBegin {
                            title: title.to_string(),
                            message: message.map(|s| s.to_string()),
                            percentage: Some(0),
                            ..Default::default()
                        },
                    )),
                })
                .unwrap(),
            }))
            .unwrap();
    }

    pub fn work_progress(&mut self, percentage: u32, message: Option<String>) {
        if percentage > self.progress || self.last_message != message {
            self.last_message = message.clone();
            self.sender
                .send(Message::Notification(Notification {
                    method: Progress::METHOD.to_string(),
                    params: serde_json::to_value(ProgressParams {
                        token: lsp_types::NumberOrString::Number(self.id as i32),
                        value: ProgressParamsValue::WorkDone(WorkDoneProgress::Report(
                            WorkDoneProgressReport {
                                percentage: Some(percentage),
                                message,
                                ..Default::default()
                            },
                        )),
                    })
                    .unwrap(),
                }))
                .unwrap();
        }
    }
    pub fn work_done(&self) {
        self.sender
            .send(Message::Notification(Notification {
                method: Progress::METHOD.to_string(),
                params: serde_json::to_value(ProgressParams {
                    token: lsp_types::NumberOrString::Number(self.id as i32),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                        WorkDoneProgressEnd {
                            ..Default::default()
                        },
                    )),
                })
                .unwrap(),
            }))
            .unwrap();
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

// TODO: Use Rust magic to automatically implement handling the requests and notifications
#[allow(unused_variables)]
pub trait LSPServer {
    type AstGenerator: ASTGenerator;

    fn handle_init_parameters(&self, params: InitializeParams) {}
    fn connection(&self) -> &LSPConnection;
    fn cache(&self) -> &Cache<Self::AstGenerator>;
    fn get_capabilities(&self) -> ServerCapabilities;

    lsp_function_req!(completion, Completion);
    lsp_function_req!(document_diagnostics, DocumentDiagnosticRequest);
    lsp_function_req!(formatting, Formatting);
    lsp_function_req!(goto_definition, GotoDefinition);
    lsp_function_req!(inlay_hint, InlayHintRequest);
    lsp_function_req!(references, References);
    lsp_function_req!(rename, Rename);
    lsp_function_req!(semantic_tokens, SemanticTokensFullRequest);
    lsp_function_req!(execute_command, ExecuteCommand);

    // Notifications

    lsp_function_not!(did_change_configuration, DidChangeConfiguration);

    fn did_close(&self, params: DidCloseTextDocumentParams) -> Result<()> {
        self.cache().remove_document(&params.text_document.uri);

        Ok(())
    }

    fn queue_diagnostics(&self, uri: &Uri);

    fn did_open(&self, params: DidOpenTextDocumentParams) -> Result<()> {
        self.cache()
            .update_content(params.text_document.uri.clone(), &params.text_document.text);
        self.queue_diagnostics(&params.text_document.uri);

        Ok(())
    }

    fn is_incremental(&self) -> bool {
        match self.get_capabilities().text_document_sync {
            Some(cap) => match cap {
                TextDocumentSyncCapability::Kind(kind) => kind == TextDocumentSyncKind::INCREMENTAL,
                TextDocumentSyncCapability::Options(options) => {
                    options.change.unwrap_or(TextDocumentSyncKind::NONE)
                        == TextDocumentSyncKind::INCREMENTAL
                }
            },
            None => false,
        }
    }

    fn did_change_text(&self, params: DidChangeTextDocumentParams) -> Result<(), LSPError> {
        for change in params.content_changes {
            let current_text = self.cache().get_document(&params.text_document.uri)?;

            let range = match change.range {
                Some(r) => r,
                None => return Err(get_response_error("Got change params without range".into())),
            };
            let mut rope = Rope::from_str(&current_text.content);
            if self.is_incremental() {
                let idx_start =
                    rope.line_to_char(range.start.line as usize) + range.start.character as usize;
                let idx_end =
                    rope.line_to_char(range.end.line as usize) + range.end.character as usize;
                rope.remove(idx_start..idx_end);
                rope.insert(idx_start, &change.text);
            } else {
                rope = Rope::from_str(&current_text.content);
            }
            let start = Instant::now();
            self.cache()
                .update_content(params.text_document.uri.clone(), rope.to_string().as_str());
            log::info!("Updating content took {:?}", start.elapsed());
            let start = Instant::now();
            for uri in self.cache().get_loaded_lsp_uris() {
                self.queue_diagnostics(&uri);
            }
            log::info!("Publishing diagnostics took {:?}", start.elapsed());
        }
        Ok(())
    }
}
