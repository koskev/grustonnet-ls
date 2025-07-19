use language_server::{server::LSPServer, utils::UriHelper};
use pretty_assertions::assert_eq;
use std::fs::{self, read_to_string};

use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, PartialResultParams, Position,
    TextDocumentIdentifier, TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};

#[derive(Default)]
pub(crate) struct DefinitionTestCase {
    pub(crate) filename: String,
    pub(crate) source: Position,
    pub(crate) target: Position,
    pub(crate) target_file: Option<String>,
}

impl DefinitionTestCase {
    fn create_server(&self) -> JsonnetServer {
        JsonnetServer {
            ..Default::default()
        }
    }

    pub(crate) fn check(&self) {
        let server = self.create_server();
        let file_content = read_to_string(&self.filename).unwrap();
        let file_uri =
            Uri::from_path(fs::canonicalize(&self.filename).unwrap().to_str().unwrap()).unwrap();
        println!("URI: {:?}", file_uri);

        server
            .cache
            .ast_generator
            .jsonnet
            .set_config(&server.configuration.read().unwrap().jsonnet);
        server
            .did_open(lsp_types::DidOpenTextDocumentParams {
                text_document: lsp_types::TextDocumentItem {
                    uri: file_uri.clone(),
                    language_id: "jsonnet".into(),
                    version: 1,
                    text: file_content.clone(),
                },
            })
            .unwrap();

        let defs = server
            .goto_definition(GotoDefinitionParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: file_uri },
                    position: self.source.clone(),
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .unwrap();
        let defs: GotoDefinitionResponse = serde_json::from_value(defs.0).unwrap();

        match defs {
            GotoDefinitionResponse::Scalar(loc) => {
                assert_eq!(
                    loc.uri.as_str(),
                    format!(
                        "file://{}",
                        fs::canonicalize(self.target_file.clone().unwrap_or(self.filename.clone()))
                            .unwrap()
                            .to_str()
                            .unwrap()
                    )
                );
                assert_eq!(loc.range.start, self.target);
            }
            _ => assert!(false, "Not supported"),
        }
    }
}

#[test]
fn simple() {
    DefinitionTestCase {
        filename: "testdata/definition/simple.jsonnet".into(),
        source: Position {
            line: 4,
            character: 5,
        },
        target: Position {
            line: 1,
            character: 6,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_same_var() {
    DefinitionTestCase {
        filename: "testdata/definition/simple.jsonnet".into(),
        source: Position {
            line: 1,
            character: 9,
        },
        target: Position {
            line: 1,
            character: 6,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn object() {
    DefinitionTestCase {
        filename: "testdata/definition/object.jsonnet".into(),
        source: Position {
            line: 5,
            character: 11,
        },
        target: Position {
            line: 1,
            character: 2,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn object_nested_var() {
    DefinitionTestCase {
        filename: "testdata/definition/object_nested.jsonnet".into(),
        source: Position {
            line: 7,
            character: 5,
        },
        target: Position {
            line: 0,
            character: 6,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn object_nested_outer() {
    DefinitionTestCase {
        filename: "testdata/definition/object_nested.jsonnet".into(),
        source: Position {
            line: 7,
            character: 11,
        },
        target: Position {
            line: 1,
            character: 2,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn object_nested_inner() {
    DefinitionTestCase {
        filename: "testdata/definition/object_nested.jsonnet".into(),
        source: Position {
            line: 7,
            character: 17,
        },
        target: Position {
            line: 2,
            character: 4,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn object_nested_inner_field() {
    DefinitionTestCase {
        filename: "testdata/definition/object_nested.jsonnet".into(),
        source: Position {
            line: 2,
            character: 8,
        },
        target: Position {
            line: 2,
            character: 4,
        },
        ..Default::default()
    }
    .check();
}

#[test]
fn import_simple() {
    DefinitionTestCase {
        filename: "testdata/definition/import_simple.jsonnet".into(),
        source: Position {
            line: 2,
            character: 9,
        },
        target: Position {
            line: 1,
            character: 2,
        },
        target_file: Some("testdata/definition/lib.libsonnet".into()),
        ..Default::default()
    }
    .check();
}

#[test]
fn local_function() {
    DefinitionTestCase {
        filename: "testdata/definition/local_function.jsonnet".into(),
        source: Position {
            line: 4,
            character: 7,
        },
        target: Position {
            line: 0,
            character: 6,
        },
        ..Default::default()
    }
    .check();
}

#[test]
#[ignore = "not implemented"]
fn local_function_arg() {
    DefinitionTestCase {
        filename: "testdata/definition/local_function.jsonnet".into(),
        source: Position {
            line: 1,
            character: 8,
        },
        target: Position {
            line: 0,
            character: 13,
        },
        ..Default::default()
    }
    .check();
}
