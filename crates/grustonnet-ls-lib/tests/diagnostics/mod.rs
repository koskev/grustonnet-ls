use language_server::{server::LSPServer, utils::UriHelper};
use pretty_assertions::assert_eq;
use std::{
    fs::read_to_string,
    sync::{Arc, RwLock},
};

pub use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Uri};

use grustonnet_ls_lib::server::{
    config::{Configuration, DiagnosticConfig},
    jsonnet::JsonnetServer,
};

pub mod empty;
pub mod error;
pub mod runtime;
pub mod r#static;

pub(crate) struct DiagnosticTestCase {
    pub(crate) filename: String,
    pub(crate) expected: Vec<Diagnostic>,
    pub(crate) config: DiagnosticConfig,
}

impl Default for DiagnosticTestCase {
    fn default() -> Self {
        Self {
            filename: String::default(),
            expected: vec![],
            // TODO: Use this default for now to not touch the tests :)
            config: DiagnosticConfig {
                enable_eval: true,
                enable_go_lint: true,
                enable_lint: false,
            },
        }
    }
}

impl DiagnosticTestCase {
    fn create_server(&self) -> JsonnetServer {
        JsonnetServer {
            configuration: Arc::new(RwLock::new(Configuration {
                diagnostics: self.config.clone(),
                ..Default::default()
            })),
            ..Default::default()
        }
    }
    pub(crate) fn check(&self) {
        let server = self.create_server();
        let file_content = read_to_string(&self.filename).unwrap();
        let file_uri = Uri::from_path(&self.filename).unwrap();

        server
            .configuration
            .write()
            .unwrap()
            .jsonnet
            .ext_code
            .insert("PARAMS".to_string(), "{}".to_string());
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

        let mut diagnositcs = server.get_diagnostics(&Uri::from_path(&self.filename).unwrap());

        diagnositcs
            .iter_mut()
            .for_each(|diag| diag.code_description = None);

        assert_eq!(diagnositcs, self.expected);
    }
}
