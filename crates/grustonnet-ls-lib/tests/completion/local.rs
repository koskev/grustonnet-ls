use grustonnet_ls_lib::server::config::{CompletionConfig, Configuration};
use lsp_types::{CompletionItem, CompletionList};

use crate::completion::completion::CompletionTestCase;

fn local_config() -> Configuration {
    Configuration {
        completion: CompletionConfig {
            enable_keywords: false,
            enable_global: false,
            enable_local: true,
        },
        ..Default::default()
    }
}

#[test]
#[ignore = "Currently not supported"]
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
    }
    .check();
}

#[test]
#[ignore = "Currently not supported"]
fn object_multiple() {
    CompletionTestCase {
        filename: "testdata/simple_object_multiple_fields.jsonnet".into(),
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
    }
    .check();
}
