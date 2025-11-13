use grustonnet_ls_lib::{
    diagnostics::{JsonnetDiagnostics, linters::dollar::DollarDiagnostics},
    server::config::DiagnosticConfig,
};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::diagnostics::{DiagnosticTestCase, IgnoreFields};

#[test]
fn in_object() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/dollar/dollar.jsonnet".to_string(),
        config: DiagnosticConfig {
            prevent_dollar: true,
            ..Default::default()
        },
        expected: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 2,
                    character: 4,
                },
                end: Position {
                    line: 2,
                    character: 5,
                },
            },
            severity: Some(DiagnosticSeverity::HINT),
            source: Some(DollarDiagnostics::default().get_name()),

            ..Default::default()
        }],
        ignore: IgnoreFields {
            message: true,
            source: false,
        },
        ..Default::default()
    }
    .check()
}
