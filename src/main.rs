use std::error::Error;

use crate::{
    node::Node,
    server::{JsonnetServer, LSPResponse, LSPServer},
};
use anyhow::{Result, anyhow};
use lsp_server::{
    Connection, ErrorCode, ExtractError, Message, Notification, Request, RequestId, Response,
    ResponseError,
};
use lsp_types::{
    CompletionList, CompletionOptions, GotoDefinitionResponse, InitializeParams,
    ServerCapabilities, TextDocumentSyncKind, TextDocumentSyncOptions,
    notification::{DidChangeConfiguration, DidChangeTextDocument},
    request::{Completion, GotoDefinition, Initialize},
};

mod ast;
mod cache;
mod node;
mod server;
mod utils;

pub mod binding {
    #![allow(warnings)]
    rust2go::r2g_include_binding!();
}

#[tokio::main]
async fn main() {
    let server = JsonnetServer::new();
    main_loop(server).unwrap()
}

fn main_loop<S: LSPServer>(server: S) -> Result<(), Box<dyn Error + Sync + Send>> {
    //let (connection, io_threads) = Connection::stdio();
    let (connection, io_threads) = Connection::listen("127.0.0.1:4874").unwrap();

    let server_capabilities = serde_json::to_value(server.get_capabilities()).unwrap();
    let params = connection
        .initialize(server_capabilities)
        .expect("init connection");

    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting example main loop");
    for msg in &connection.receiver {
        eprintln!("got msg: {msg:?}");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {req:?}");
                let resp = handle_request(req.clone(), &server);
                let result: Result<serde_json::Value, ResponseError> = match resp {
                    Ok(val) => Ok(val.into()),
                    Err(e) => Err(e),
                };

                eprintln!("Sending response {:?}", result);

                connection.sender.send(Message::Response(Response {
                    id: req.id,
                    result: result.clone().ok(),
                    error: result.err(),
                }))?
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(not) => {
                let _ = handle_notification(not.clone(), &server);
            }
        }
    }
    io_threads.join().unwrap();
    Ok(())
}

fn handle_notification<S: LSPServer>(req: Notification, server: &S) -> Result<(), ResponseError> {
    match cast_notification::<DidChangeConfiguration>(req.clone()) {
        Ok(params) => match server.did_change_configuration(params) {
            Ok(()) => eprintln!("Config changed"),
            Err(err) => eprintln!("Notification error: {}", err),
        },
        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        Err(ExtractError::MethodMismatch(_req)) => (),
    };
    match cast_notification::<DidChangeTextDocument>(req) {
        Ok(params) => match server.did_change_text(params) {
            Ok(()) => (),
            Err(err) => eprintln!("Notification error: {}", err),
        },
        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        Err(ExtractError::MethodMismatch(_req)) => (),
    };

    Err(ResponseError {
        code: ErrorCode::MethodNotFound as i32,
        message: "Method not implemented".into(),
        data: None,
    })
}

fn handle_request<S: LSPServer>(req: Request, server: &S) -> Result<LSPResponse, ResponseError> {
    let req = match cast_req::<Initialize>(req) {
        Ok((_id, params)) => {
            let resp = server.initialize(params);
            eprintln!("Initialized server!");
            return resp;
        }
        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        Err(ExtractError::MethodMismatch(req)) => req,
    };
    match cast_req::<Completion>(req) {
        Ok((_id, params)) => {
            eprintln!("äääääääääääääääää");
            let resp = server.completion(params);
            return resp;
        }
        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        Err(ExtractError::MethodMismatch(req)) => req,
    };

    Err(ResponseError {
        code: ErrorCode::MethodNotFound as i32,
        message: "Method not implemented".into(),
        data: None,
    })
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
