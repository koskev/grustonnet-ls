use super::*;

#[test]
fn shadow_first() {
    CompletionTestCase {
        filename: "testdata/complete/local/shadow.jsonnet".into(),
        replace_string: "x: myVar".into(),
        replace_by_string: "x: myVar.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "two".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn shadow_nested() {
    CompletionTestCase {
        filename: "testdata/complete/local/shadow.jsonnet".into(),
        replace_string: "x: myVar".into(),
        replace_by_string: "x: myVar.two.".into(),
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
