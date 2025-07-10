use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::diagnostics::DiagnosticTestCase;

#[test]
fn empty() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/static/empty.jsonnet".to_string(),
        expected: vec![
            Diagnostic {
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unexpected end of file".to_string(),
                ..Default::default()
            },
            Diagnostic {
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Unexpected end of file".to_string(),
                ..Default::default()
            },
        ],
    }
    .check()
}

#[test]
fn syntax() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/static/syntax.jsonnet".to_string(),
        expected: vec![
            Diagnostic {
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Expected a comma before next field".to_string(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 1,
                    },
                    end: Position {
                        line: 2,
                        character: 1,
                    },
                },
                ..Default::default()
            },
            Diagnostic {
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Expected a comma before next field".to_string(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 1,
                    },
                    end: Position {
                        line: 2,
                        character: 1,
                    },
                },
                ..Default::default()
            },
        ],
    }
    .check()
}

#[test]
fn unused() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/static/unused.jsonnet".to_string(),
        expected: vec![Diagnostic {
            severity: Some(DiagnosticSeverity::WARNING),
            range: Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 6,
                },
            },
            message: "Unused variable: a".to_string(),
            ..Default::default()
        }],
    }
    .check()
}
