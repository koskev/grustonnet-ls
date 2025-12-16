use grustonnet_config::DiagnosticConfig;
use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, Position, Range};

use crate::diagnostics::{DiagnosticTestCase, disabled_diagnostics_config};

const ERROR_MESSAGE: &str = "Unused variable. If this is intentional prefix with an underscore: _";

#[test]
fn local_var() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/local_var.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: true,
            ..disabled_diagnostics_config()
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
            ..disabled_diagnostics_config()
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
fn object_local_var() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/object_local_var.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: true,
            ..disabled_diagnostics_config()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 7,
                },
                end: Position {
                    line: 1,
                    character: 16,
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
