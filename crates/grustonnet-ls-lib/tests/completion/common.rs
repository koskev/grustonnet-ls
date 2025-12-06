use std::{
    collections::HashMap,
    fs::read_to_string,
    sync::{Arc, Once, RwLock},
};

use grustonnet_config::Configuration;
use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use language_server::{
    server::LSPServer,
    utils::{UriHelper, rope::RopeHelper},
};
use lsp_types::{
    CompletionList, PartialResultParams, Position, Range, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};
use pretty_assertions::assert_eq;
use ropey::Rope;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[derive(Default)]
pub(crate) struct CompletionTestCase {
    pub(crate) filename: String,
    pub(crate) replace_string: String,
    pub(crate) replace_by_string: String,
    pub(crate) expected: CompletionList,

    pub(crate) config: Configuration,

    pub(crate) ext_code: HashMap<String, String>,
}

impl CompletionTestCase {
    fn create_server(&self) -> JsonnetServer {
        JsonnetServer {
            configuration: Arc::new(RwLock::new(self.config.clone())),
            ..Default::default()
        }
    }

    pub(crate) fn get_completions(&self) -> (CompletionList, Rope, Position) {
        setup();
        let server = self.create_server();
        let file_content = read_to_string(&self.filename)
            .unwrap_or_else(|_| panic!("{} not found", self.filename));
        let file_uri = Uri::from_path(&self.filename).unwrap();
        server.configuration.write().unwrap().jsonnet.ext_code = self.ext_code.clone();
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

        let string_begin = file_content
            .clone()
            .find(&self.replace_string)
            .expect("Unable to find string");
        let string_end = string_begin + self.replace_string.len();
        let mut rope = Rope::from_str(&file_content);

        server
            .did_change_text(lsp_types::DidChangeTextDocumentParams {
                text_document: lsp_types::VersionedTextDocumentIdentifier {
                    uri: file_uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    text: self.replace_by_string.clone(),
                    range: Some(Range {
                        start: rope.get_location(string_begin).unwrap(),
                        end: rope.get_location(string_end).unwrap(),
                    }),
                    range_length: None,
                }],
            })
            .unwrap();
        let completion_location = rope
            .replace_get_end(&self.replace_string, &self.replace_by_string)
            .unwrap();

        let completion_list = server
            .completion(lsp_types::CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: file_uri },
                    position: completion_location,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                context: None,
                partial_result_params: PartialResultParams::default(),
            })
            .unwrap();
        (
            serde_json::from_value(completion_list.0).unwrap(),
            rope,
            completion_location,
        )
    }

    pub(crate) fn check(&self) {
        let (mut completion_list, rope, completion_location) = self.get_completions();
        for (item, expected) in completion_list
            .items
            .iter_mut()
            .zip(self.expected.items.iter())
        {
            if expected.detail.is_none() {
                item.detail = None;
            }
            // If kind was not set in the test we don't test it
            if expected.kind.is_none() {
                item.kind = None;
            }
            if expected.label_details.is_none() {
                item.label_details = None;
            }
        }

        assert_eq!(
            self.expected,
            completion_list,
            "At {:?} with \n{}",
            completion_location,
            rope.lines()
                .enumerate()
                .map(|(line_num, line)| {
                    format!("{}({}):{}", line_num + 1, line.len_chars(), line)
                })
                .collect::<String>()
        )
    }
}
