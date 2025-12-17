use grustonnet_config::{DiagnosticConfig, UnusedVariablesConfig};
use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, Position, Range};

use crate::diagnostics::{DiagnosticTestCase, disabled_diagnostics_config};

const ERROR_MESSAGE: &str = "Unused variable. If this is intentional prefix with an underscore: _";

#[test]
fn local_var() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/local_var.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: UnusedVariablesConfig::default(),
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
            unused_variables: UnusedVariablesConfig::default(),
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
            unused_variables: UnusedVariablesConfig::default(),
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

#[test]
fn local_func_unused() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/local_func_unused.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: UnusedVariablesConfig::default(),
            ..disabled_diagnostics_config()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 13,
                },
                end: Position {
                    line: 0,
                    character: 16,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("{}{}", ERROR_MESSAGE, "arg"),
            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
            ..Default::default()
        }],
        ..Default::default()
    }
    .check()
}

#[test]
//#[ignore = "not implemented"]
fn object_func_unused() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/unused/object_func_unused.jsonnet".to_string(),
        config: DiagnosticConfig {
            unused_variables: UnusedVariablesConfig::default(),
            ..disabled_diagnostics_config()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 4,
                },
                end: Position {
                    line: 1,
                    character: 7,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("{}{}", ERROR_MESSAGE, "arg"),
            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
            ..Default::default()
        }],
        ..Default::default()
    }
    .check()
}
