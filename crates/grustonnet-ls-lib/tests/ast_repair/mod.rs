use language_server::{cache::ASTState, server::LSPServer, utils::UriHelper};
use pretty_assertions::assert_eq;
use std::fs::{self, read_to_string};

use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use lsp_types::Uri;

#[derive(Default)]
pub(crate) struct AstRepairTestCase {
    pub(crate) filename: String,
}

impl AstRepairTestCase {
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

        assert_eq!(
            ASTState::Clean,
            server.cache.get_document(&file_uri).unwrap().state
        );
    }
}

fn check_ast_repair(val: &str) {
    AstRepairTestCase {
        filename: val.to_string(),
    }
    .check();
}

test_macros::generate_test_function_for_dir!("testdata/ast_repair/", check_ast_repair);
