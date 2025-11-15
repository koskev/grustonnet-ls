use grustonnet_config::{CompletionConfig, Configuration};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};

use crate::completion::common::CompletionTestCase;

fn global_config() -> Configuration {
    Configuration {
        completion: CompletionConfig {
            enable_keywords: false,
            enable_global: true,
            enable_local: false,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn simple_local() {
    CompletionTestCase {
        filename: "testdata/simple_local.jsonnet".into(),
        replace_string: "x: myVar,".into(),
        replace_by_string: "x: my".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "myVar".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            }],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_local_func() {
    CompletionTestCase {
        filename: "testdata/simple_local_func.jsonnet".into(),
        replace_string: "x: myFunc(),".into(),
        replace_by_string: "x: my".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "myFunc".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            }],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn function_args() {
    CompletionTestCase {
        filename: "testdata/complete/functions/function_defaults.jsonnet".into(),
        replace_string: "x: argone,".into(),
        replace_by_string: "x: ".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "myFunc".to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()
                },
                CompletionItem {
                    label: "argone".to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    ..Default::default()
                },
                CompletionItem {
                    label: "argtwo".to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
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
fn inside_array_top() {
    CompletionTestCase {
        filename: "testdata/complete/array/inside_array.jsonnet".into(),
        replace_string: "// 1".into(),
        replace_by_string: "my".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "myVar".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            }],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn inside_array_middle() {
    CompletionTestCase {
        filename: "testdata/complete/array/inside_array.jsonnet".into(),
        replace_string: "// 2".into(),
        replace_by_string: "my".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "myVar".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            }],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn inside_array_bottom() {
    CompletionTestCase {
        filename: "testdata/complete/array/inside_array.jsonnet".into(),
        replace_string: "// 3".into(),
        replace_by_string: "my".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "myVar".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            }],
        },
        config: global_config(),
        ..Default::default()
    }
    .check();
}
