use grustonnet_ls_lib::server::config::DiagnosticConfig;
use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, Position, Range};

use crate::diagnostics::DiagnosticTestCase;

const ERROR_MESSAGE: &str = "Unused variable. If this is intentional prefix with an underscore: _";

#[test]
fn local_var() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/local_var.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: true,
            ..Default::default()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 15,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("{}{}", ERROR_MESSAGE, "myVar"),
            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
            ..Default::default()
        }],
        ..Default::default()
    }
    .check()
}

#[test]
fn local_func() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/local_func.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: true,
            ..Default::default()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 24,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("{}{}", ERROR_MESSAGE, "myFunc"),
            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
            ..Default::default()
        }],
        ..Default::default()
    }
    .check()
}

#[test]
#[ignore = "not implemented"]
fn object_local_var() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/object_local_var.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: true,
            ..Default::default()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 6,
                },
                end: Position {
                    line: 1,
                    character: 15,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("{}{}", ERROR_MESSAGE, "myVar"),
            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
            ..Default::default()
        }],
        ..Default::default()
    }
    .check()
}
