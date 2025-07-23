use super::*;

#[test]
fn std_without_loop_support() {
    CompletionTestCase {
        filename: "testdata/complete/std/loop.jsonnet".into(),
        replace_string: "x: combined".into(),
        replace_by_string: "x: combined.".into(),
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
