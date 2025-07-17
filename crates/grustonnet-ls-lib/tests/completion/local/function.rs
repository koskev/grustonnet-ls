use super::*;

#[test]
fn complete_in_args_positional() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args.jsonnet".into(),
        replace_string: "myObj.objKey".into(),
        replace_by_string: "myObj.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_named() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args.jsonnet".into(),
        replace_string: "myObj.objKey".into(),
        replace_by_string: "arg = myObj.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
