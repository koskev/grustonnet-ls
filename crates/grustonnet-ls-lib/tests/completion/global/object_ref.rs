use grustonnet_config::{CompletionConfig, Configuration, ObjectRef};
use lsp_types::{CompletionItem, CompletionList};

use crate::completion::{common::CompletionTestCase, disabled_config};

fn global_config() -> Configuration {
    Configuration {
        completion: CompletionConfig {
            object_ref: ObjectRef {
                enabled: true,
                ..Default::default()
            },
            ..disabled_config()
        },
        ..Default::default()
    }
}

#[test]
fn simple_local_file_inner() {
    CompletionTestCase {
        filename: "testdata/complete/object_ref/local_file_inner.jsonnet".into(),
        replace_string: "// x".into(),
        replace_by_string: "".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "val1".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "val2".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_local_file_outer() {
    CompletionTestCase {
        filename: "testdata/complete/object_ref/local_file_outer.jsonnet".into(),
        replace_string: "// x".into(),
        replace_by_string: "".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "val1".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "val2".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}

#[ignore = "Requires network access"]
#[test]
fn simple_remote_json() {
    CompletionTestCase {
        filename: "testdata/complete/object_ref/url_simple.jsonnet".into(),
        replace_string: "// x".into(),
        replace_by_string: "".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "properties".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "title".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "type".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}
