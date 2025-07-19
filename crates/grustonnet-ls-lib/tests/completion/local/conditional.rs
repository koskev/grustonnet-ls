use super::*;

#[test]
fn inside_cond() {
    CompletionTestCase {
        filename: "testdata/complete/conditional/inside.jsonnet".into(),
        replace_string: "myObj == 4".into(),
        replace_by_string: "myObj.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "keyTrue".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "keyFalse".to_string(),
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
fn inside_true() {
    CompletionTestCase {
        filename: "testdata/complete/conditional/inside.jsonnet".into(),
        replace_string: "myObj.keyTrue".into(),
        replace_by_string: "myObj.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "keyTrue".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "keyFalse".to_string(),
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
fn inside_false() {
    CompletionTestCase {
        filename: "testdata/complete/conditional/inside.jsonnet".into(),
        replace_string: "myObj.keyFalse".into(),
        replace_by_string: "myObj.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "keyTrue".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "keyFalse".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
