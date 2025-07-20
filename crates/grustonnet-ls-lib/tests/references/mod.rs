use language_server::{server::LSPServer, utils::UriHelper};
use pretty_assertions::assert_eq;
use std::fs::{self, read_to_string};

use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use lsp_types::{
    Location, PartialResultParams, Position, Range, ReferenceContext, ReferenceParams,
    TextDocumentIdentifier, TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};

#[derive(Default, Debug)]
pub(crate) struct TargetInfo {
    range: Range,
    uri: Option<Uri>,
}

#[derive(Default)]
pub(crate) struct ReferenceTestCase {
    pub(crate) filename: String,
    pub(crate) source: Position,
    pub(crate) targets: Vec<TargetInfo>,
}

impl ReferenceTestCase {
    fn create_server(&self) -> JsonnetServer {
        JsonnetServer {
            ..Default::default()
        }
    }

    pub(crate) fn check(&self) {
        let server = self.create_server();
        *server.cache.ast_generator.jsonnet.root_dir.write().unwrap() = ".".into();
        let file_uri = Uri::from_path(
            fs::canonicalize(&self.filename)
                .expect("Failed to get full path")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let file_content = read_to_string(&self.filename)
            .unwrap_or_else(|_| panic!("Failed to read {}", self.filename));

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

        let references = server
            .references(ReferenceParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: file_uri },
                    position: self.source,
                },
                context: ReferenceContext {
                    include_declaration: true,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .unwrap();
        let references = serde_json::from_value::<Option<Vec<Location>>>(references.0)
            .unwrap()
            .expect("No results");

        let targets: Vec<_> = self
            .targets
            .iter()
            .map(|info| Location {
                uri: info
                    .uri
                    .clone()
                    .unwrap_or(Uri::from_path(&self.filename).unwrap()),
                range: info.range,
            })
            .collect();

        assert_eq!(references, targets);
    }
}

#[test]
fn simple() {
    ReferenceTestCase {
        filename: "testdata/references/simple.jsonnet".into(),
        source: Position {
            line: 5,
            character: 7,
        },
        targets: vec![
            TargetInfo {
                range: Range {
                    start: Position {
                        line: 1,
                        character: 6,
                    },
                    end: Position {
                        line: 1,
                        character: 10,
                    },
                },
                ..Default::default()
            },
            TargetInfo {
                range: Range {
                    start: Position {
                        line: 5,
                        character: 5,
                    },
                    end: Position {
                        line: 5,
                        character: 9,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
    .check();
}

#[test]
fn object() {
    ReferenceTestCase {
        filename: "testdata/references/object.jsonnet".into(),
        source: Position {
            line: 5,
            character: 12,
        },
        targets: vec![
            TargetInfo {
                range: Range {
                    start: Position {
                        line: 1,
                        character: 2,
                    },
                    end: Position {
                        line: 1,
                        character: 5,
                    },
                },
                ..Default::default()
            },
            TargetInfo {
                range: Range {
                    start: Position {
                        line: 5,
                        character: 11,
                    },
                    end: Position {
                        line: 5,
                        character: 14,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
    .check();
}
