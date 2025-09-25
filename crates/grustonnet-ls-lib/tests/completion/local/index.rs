use super::*;

#[test]
fn var_index() {
    CompletionTestCase {
        filename: "testdata/complete/object/object_computed_index.jsonnet".into(),
        replace_string: "x: myObject[firstVar]".into(),
        replace_by_string: "x: myObject[firstVar].".into(),
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
#[ignore = "weird bug where the document stack does not contain the last local"]
fn var_index_var_last() {
    CompletionTestCase {
        filename: "testdata/complete/object/object_computed_index_var_last.jsonnet".into(),
        replace_string: "x: myObject[firstVar]".into(),
        replace_by_string: "x: myObject[firstVar].".into(),
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
