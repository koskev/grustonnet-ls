use assert_unordered::assert_eq_unordered;
use language_server::{server::LSPServer, utils::UriHelper};
use std::{
    fs::read_to_string,
    sync::{Arc, RwLock},
};

pub use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Uri};

use grustonnet_ls_lib::server::{
    config::{Configuration, DiagnosticConfig, DuplicateDetectionConfig, VariableNaming},
    jsonnet::JsonnetServer,
};

pub mod empty;
pub mod error;
pub mod runtime;
pub mod snake;
pub mod r#static;
pub mod unused;

#[derive(Default)]
pub struct IgnoreFields {
    message: bool,
    source: bool,
}

pub(crate) struct DiagnosticTestCase {
    pub(crate) filename: String,
    pub(crate) expected: Vec<Diagnostic>,
    pub(crate) config: DiagnosticConfig,
    pub(crate) ignore: IgnoreFields,
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
                unused_variables: false,
                variable_naming: VariableNaming::None,
                local_function: false,
                prevent_dollar: false,
                recursive_arguments: false,
                shadow_variable: false,
                duplicate_detection: DuplicateDetectionConfig {
                    min_occurrences: 0,
                    ..Default::default()
                },
            },
            ignore: IgnoreFields {
                message: false,
                source: true,
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

        let diagnostics = server.get_diagnostics(&Uri::from_path(&self.filename).unwrap());

        let diagnostics: Vec<Diagnostic> = diagnostics
            .into_iter()
            .map(|d| d.diagnostics)
            .map(|mut diag| {
                diag.code_description = None;
                if self.ignore.message {
                    diag.message = "".into();
                }
                if self.ignore.source {
                    diag.source = Some("".into());
                }
                diag
            })
            .collect();
        // TODO: macro to remove duplicate logic
        let expected = self
            .expected
            .clone()
            .into_iter()
            .map(|mut diag| {
                if self.ignore.message {
                    diag.message = "".into();
                }
                if self.ignore.source {
                    diag.source = Some("".into());
                }
                diag
            })
            .collect();

        assert_eq_unordered!(diagnostics, expected);
    }
}
