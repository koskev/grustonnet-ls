use std::str::FromStr;

use grustonnet_ls_lib::{
    diagnostics::{JsonnetDiagnostics, linters::shadow_variable::ShadowVariableDiagnostics},
    server::config::DiagnosticConfig,
};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, Position, Range, Uri,
};

use crate::diagnostics::{DiagnosticTestCase, IgnoreFields};

#[test]
fn local_var() {
    let first_loc = Range {
        start: Position {
            line: 1,
            character: 6,
        },
        end: Position {
            line: 1,
            character: 16,
        },
    };
    let second_loc = Range {
        start: Position {
            line: 0,
            character: 6,
        },
        end: Position {
            line: 0,
            character: 16,
        },
    };
    DiagnosticTestCase {
        filename: "testdata/diagnostics/shadow/local_var.jsonnet".to_string(),
        config: DiagnosticConfig {
            shadow_variable: true,
            ..Default::default()
        },
        expected: vec![
            Diagnostic {
                range: first_loc,
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some(ShadowVariableDiagnostics::default().get_name()),
                related_information: Some(vec![DiagnosticRelatedInformation {
                    location: Location {
                        uri: Uri::from_str("file").unwrap(),
                        range: second_loc,
                    },
                    message: "".into(),
                }]),

                ..Default::default()
            },
            Diagnostic {
                range: second_loc,
                severity: Some(DiagnosticSeverity::INFORMATION),
                source: Some(ShadowVariableDiagnostics::default().get_name()),
                related_information: Some(vec![DiagnosticRelatedInformation {
                    location: Location {
                        uri: Uri::from_str("file").unwrap(),
                        range: first_loc,
                    },
                    message: "".into(),
                }]),

                ..Default::default()
            },
        ],
        ignore: IgnoreFields {
            message: true,
            source: false,
        },
        ..Default::default()
    }
    .check()
}

#[test]
#[ignore = "not implementd"]
fn local_var_obj() {
    let first_loc = Range {
        start: Position {
            line: 2,
            character: 6,
        },
        end: Position {
            line: 2,
            character: 16,
        },
    };
    let second_loc = Range {
        start: Position {
            line: 0,
            character: 6,
        },
        end: Position {
            line: 0,
            character: 16,
        },
    };
    DiagnosticTestCase {
        filename: "testdata/diagnostics/shadow/local_obj.jsonnet".to_string(),
        config: DiagnosticConfig {
            shadow_variable: true,
            ..Default::default()
        },
        expected: vec![
            Diagnostic {
                range: first_loc,
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some(ShadowVariableDiagnostics::default().get_name()),
                related_information: Some(vec![DiagnosticRelatedInformation {
                    location: Location {
                        uri: Uri::from_str("file").unwrap(),
                        range: second_loc,
                    },
                    message: "".into(),
                }]),

                ..Default::default()
            },
            Diagnostic {
                range: second_loc,
                severity: Some(DiagnosticSeverity::INFORMATION),
                source: Some(ShadowVariableDiagnostics::default().get_name()),
                related_information: Some(vec![DiagnosticRelatedInformation {
                    location: Location {
                        uri: Uri::from_str("file").unwrap(),
                        range: first_loc,
                    },
                    message: "".into(),
                }]),

                ..Default::default()
            },
        ],
        ignore: IgnoreFields {
            message: true,
            source: false,
        },
        ..Default::default()
    }
    .check()
}
