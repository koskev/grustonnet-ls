use std::collections::HashMap;

use super::*;
#[test]
fn ext_code_simple() {
    CompletionTestCase {
        filename: "testdata/complete/extvar/extcode.jsonnet".into(),
        replace_string: "x: params".into(),
        replace_by_string: "x: params.".into(),
        ext_code: HashMap::from([("PARAMS".into(), "{a: 1, b: 2}".into())]),
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
