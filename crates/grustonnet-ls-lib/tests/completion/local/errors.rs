use super::*;

#[test]
fn dollar_simple() {
    CompletionTestCase {
        filename: "testdata/error/binary.jsonnet".into(),
        replace_string: "x: myVar".into(),
        replace_by_string: "x: myVar.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "key".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
