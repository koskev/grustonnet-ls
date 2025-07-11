use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::diagnostics::DiagnosticTestCase;

#[test]
fn array() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/runtime/array.jsonnet".to_string(),
        expected: vec![Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Index 5 out of bounds, not within [0, 0)".to_string(),
            range: Range {
                start: Position {
                    line: 3,
                    character: 5,
                },
                end: Position {
                    line: 3,
                    character: 11,
                },
            },
            ..Default::default()
        }],
    }
    .check()
}

#[test]
fn assert() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/runtime/assert.jsonnet".to_string(),
        expected: vec![Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Assertion failed".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 2,
                },
            },
            ..Default::default()
        }],
    }
    .check()
}

#[test]
fn assert_nested() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/runtime/assert_nested.jsonnet".to_string(),
        expected: vec![Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Object assertion failed.".to_string(),
            range: Range {
                start: Position {
                    line: 3,
                    character: 6,
                },
                end: Position {
                    line: 3,
                    character: 18,
                },
            },
            ..Default::default()
        }],
    }
    .check()
}

#[test]
fn assert_function() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/runtime/assert_function.jsonnet".to_string(),
        expected: vec![Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Object assertion failed.".to_string(),
            range: Range {
                start: Position {
                    line: 1,
                    character: 2,
                },
                end: Position {
                    line: 1,
                    character: 26,
                },
            },
            ..Default::default()
        }],
    }
    .check()
}
