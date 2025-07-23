use super::*;

#[test]
fn error_circular_import() {
    DiagnosticTestCase {
        filename: "testdata/diagnostics/error_cases/circular.jsonnet".to_string(),
        expected: vec![
            Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            message: "unknown error: RUNTIME ERROR: max stack frames exceeded.\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\t...\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tField \"x\"\t\n\tDuring manifestation\t\n".to_string(),
            ..Default::default()
        },
            Diagnostic {
            severity: Some(DiagnosticSeverity::WARNING),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            message: "unknown error: GO Error: Bug - placeholder for a dependent node cannot be noType".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
    .check()
}
