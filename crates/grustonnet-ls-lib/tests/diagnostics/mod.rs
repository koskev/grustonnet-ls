use language_server::server::LSPServer;
use pretty_assertions::assert_eq;
use std::{fs::read_to_string, str::FromStr};

use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use lsp_types::{Diagnostic, Uri};

pub mod r#static;

#[derive(Default)]
pub(crate) struct DiagnosticTestCase {
    pub(crate) filename: String,
    pub(crate) expected: Vec<Diagnostic>,
}

impl DiagnosticTestCase {
    fn create_server(&self) -> JsonnetServer {
        JsonnetServer {
            ..Default::default()
        }
    }
    pub(crate) fn check(&self) {
        let server = self.create_server();
        let file_content = read_to_string(&self.filename).unwrap();
        let file_uri = Uri::from_str(&self.filename).unwrap();
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

        let mut diagnositcs = server.get_diagnostics(&self.filename);

        diagnositcs
            .iter_mut()
            .for_each(|diag| diag.code_description = None);

        assert_eq!(diagnositcs, self.expected);
    }
}
