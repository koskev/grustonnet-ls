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

#[test]
fn complete_in_args_named_multiple_first() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "arg1=myObj1.objKey1".into(),
        replace_by_string: "arg1 = myObj1.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey1".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_named_multiple_second() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "arg2=myObj2.objKey2".into(),
        replace_by_string: "arg2 = myObj2.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey2".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_multiple_third() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "arg3=myObj3.objKey3".into(),
        replace_by_string: "arg3 = myObj3.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey3".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_positional_first() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "myObj1.objKey1, // 1".into(),
        replace_by_string: "myObj1.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey1".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_positional_second() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "myObj2.objKey2, // 2".into(),
        replace_by_string: "myObj2.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey2".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_positional_third() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "myObj3.objKey3, // 3".into(),
        replace_by_string: "myObj3.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey3".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn complete_in_args_named_positiona_mixed() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_complete_in_args_multiple.jsonnet".into(),
        replace_string: "myObj3.objKey3, // 3".into(),
        replace_by_string: "arg3=myObj3.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "objKey3".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
