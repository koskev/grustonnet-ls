use super::*;

#[test]
fn function_in_object_without_arg() {
    CompletionTestCase {
        filename: "testdata/complete/object/local_from_obj_func.jsonnet".into(),
        replace_string: "funcKey():: myLocal".into(),
        replace_by_string: "funcKey():: myLocal.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "localKey".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn function_in_object_with_arg() {
    CompletionTestCase {
        filename: "testdata/complete/object/local_from_obj_func.jsonnet".into(),
        replace_string: "funcKey2(myArg):: myLocal".into(),
        replace_by_string: "funcKey2(myArg):: myLocal.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "localKey".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
