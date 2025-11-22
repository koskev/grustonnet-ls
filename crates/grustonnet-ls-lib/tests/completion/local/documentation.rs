use super::*;


#[test]
fn docsonnet_func() {
    CompletionTestCase {
        filename: "testdata/complete/docsonnet/func.jsonnet".into(),
        replace_string: "x: self.funcs.myFunc()".into(),
        replace_by_string: "x: self.funcs.my".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "myFunc".to_string(),
                    detail: Some("My Function\n".into()),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
