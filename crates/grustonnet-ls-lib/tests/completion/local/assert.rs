use super::*;

#[test]
fn simple_local() {
    CompletionTestCase {
        filename: "testdata/simple_object.jsonnet".into(),
        replace_string: "x: object,".into(),
        replace_by_string: "x: object.k".into(),
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

#[test]
fn simple_local_no_text() {
    CompletionTestCase {
        filename: "testdata/simple_object.jsonnet".into(),
        replace_string: "x: object,".into(),
        replace_by_string: "x: object.".into(),
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

#[test]
fn object_multiple_no_text() {
    CompletionTestCase {
        filename: "testdata/simple_object_multiple_fields.jsonnet".into(),
        replace_string: "x: object,".into(),
        replace_by_string: "x: object.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "key".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "second".to_string(),
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
fn object_multiple() {
    CompletionTestCase {
        filename: "testdata/simple_object_multiple_fields.jsonnet".into(),
        replace_string: "x: object,".into(),
        replace_by_string: "x: object.k".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                // All values should be completed to allow for client fuzy to find everything
                CompletionItem {
                    label: "key".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "second".to_string(),
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
fn object_nested() {
    CompletionTestCase {
        filename: "testdata/object_nested.jsonnet".into(),
        replace_string: "x: object,".into(),
        replace_by_string: "x: object.outer.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "inner".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_import() {
    CompletionTestCase {
        filename: "testdata/import/simple_import.jsonnet".into(),
        replace_string: "x: imp".into(),
        replace_by_string: "x: imp.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "importedkey".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "imported_object".to_string(),
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
fn simple_import_object() {
    CompletionTestCase {
        filename: "testdata/import/simple_import.jsonnet".into(),
        replace_string: "x: imp".into(),
        replace_by_string: "x: imp.imported_object.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "inner_obj".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn default_function_arg() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_defaults.jsonnet".into(),
        replace_string: "y: argtwo,".into(),
        replace_by_string: "y: argtwo.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "argkey".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn function_return_arg_ignored() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_return_arg_ignored.jsonnet".into(),
        replace_string: "x: myFunc(1)".into(),
        replace_by_string: "x: myFunc(1).".into(),
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

#[test]
fn function_return_arg_single() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_return_arg_single.jsonnet".into(),
        replace_string: "x: myFunc(1)".into(),
        replace_by_string: "x: myFunc({myArg: 3}).key.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "myArg".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn function_return_arg_index() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_in_object.jsonnet".into(),
        replace_string: "x: myObj".into(),
        replace_by_string: "x: myObj.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
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
fn function_return_arg_index_no_arg() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_in_object.jsonnet".into(),
        replace_string: "x: myObj".into(),
        replace_by_string: "x: myObj.withoutArg().".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "b".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn function_return_arg_index_unnamed_arg() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_in_object.jsonnet".into(),
        replace_string: "x: myObj".into(),
        replace_by_string: "x: myObj.withArg(1).".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "a".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

// TODO: macro to test all levels
#[test]
fn nested_object() {
    CompletionTestCase {
        filename: "testdata/complete/object/nested.jsonnet".into(),
        replace_string: "x: myObj".into(),
        replace_by_string: "x: myObj.one.two.three.four.five.six.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "seven".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn nested_object_func() {
    CompletionTestCase {
        filename: "testdata/complete/object/nested_with_functions.jsonnet".into(),
        replace_string: "x: myObj".into(),
        replace_by_string: "x: myObj.one.two().three.four.five().six.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "seven".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
