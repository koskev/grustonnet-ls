use grustonnet_ls_lib::server::config::{CompletionConfig, Configuration};
use lsp_types::{CompletionItem, CompletionList};

use crate::completion::completion::CompletionTestCase;

const LOCAL_CONFIG: Configuration = Configuration {
    completion: CompletionConfig {
        enable_keywords: false,
        enable_global: false,
        enable_local: true,
    },
};

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
        config: LOCAL_CONFIG.clone(),
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
        config: LOCAL_CONFIG.clone(),
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
        config: LOCAL_CONFIG.clone(),
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
        config: LOCAL_CONFIG.clone(),
    }
    .check();
}

#[test]
#[ignore = "Broken since fixing ast is not supported"]
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
        config: LOCAL_CONFIG.clone(),
    }
    .check();
}
