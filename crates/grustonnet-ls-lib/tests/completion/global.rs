use grustonnet_ls_lib::server::config::{CompletionConfig, Configuration};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};

use crate::completion::completion::CompletionTestCase;

const GLOBAL_CONFIG: Configuration = Configuration {
    completion: CompletionConfig {
        enable_keywords: false,
        enable_global: true,
        enable_local: false,
    },
};

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
        config: GLOBAL_CONFIG.clone(),
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
        config: GLOBAL_CONFIG.clone(),
    }
    .check();
}
