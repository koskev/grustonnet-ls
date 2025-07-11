use super::*;

#[test]
fn import_simple() {
    CompletionTestCase {
        filename: "testdata/complete/import/simple.jsonnet".into(),
        replace_string: "x: imported".into(),
        replace_by_string: "x: imported.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "libval".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn import_direct_access() {
    CompletionTestCase {
        filename: "testdata/complete/import/access.jsonnet".into(),
        replace_string: "x: (import 'lib.libsonnet')".into(),
        replace_by_string: "x: (import 'lib.libsonnet').".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "libval".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn import_chained() {
    CompletionTestCase {
        filename: "testdata/complete/import/chained.jsonnet".into(),
        replace_string: "x: chained".into(),
        replace_by_string: "x: chained.two.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "one".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
