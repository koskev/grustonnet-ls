use super::*;

#[test]
#[ignore = "not implemented"]
fn super_binary_simple() {
    CompletionTestCase {
        filename: "testdata/complete/super/binary.jsonnet".into(),
        replace_string: "b: super.a".into(),
        replace_by_string: "b: super.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "a".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "b".to_string(),
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
#[ignore = "not implemented"]
fn super_binary_multiple() {
    CompletionTestCase {
        filename: "testdata/complete/super/multiple_binary.jsonnet".into(),
        replace_string: "c: super.a".into(),
        replace_by_string: "b: super.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "a".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "b".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "c".to_string(),
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
#[ignore = "not implemented"]
fn super_binary_nested() {
    CompletionTestCase {
        filename: "testdata/complete/super/nested_binary.jsonnet".into(),
        replace_string: "c: super.a".into(),
        replace_by_string: "b: super.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "a".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "b".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "c".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
