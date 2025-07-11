use super::*;

#[test]
fn binary_simple() {
    CompletionTestCase {
        filename: "testdata/complete/binary/simple.jsonnet".into(),
        replace_string: "x: a".into(),
        replace_by_string: "x: a.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "two".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "one".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
#[ignore = "Currently unsupported"]
fn binary_override_single() {
    CompletionTestCase {
        filename: "testdata/complete/binary/override.jsonnet".into(),
        replace_string: "x: a".into(),
        replace_by_string: "x: a.".into(),
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

#[test]
fn binary_override_value() {
    CompletionTestCase {
        filename: "testdata/complete/binary/override.jsonnet".into(),
        replace_string: "x: a".into(),
        replace_by_string: "x: a.one.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "second".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn binary_multiple() {
    CompletionTestCase {
        filename: "testdata/complete/binary/multiple.jsonnet".into(),
        replace_string: "x: a".into(),
        replace_by_string: "x: a.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "four".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "three".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "two".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "one".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
