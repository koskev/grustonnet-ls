use lsp_types::{CompletionItem, CompletionList};

use crate::completion::{common::CompletionTestCase, local::local_config};

#[test]
fn foldl_basic() {
    CompletionTestCase {
        filename: "testdata/complete/std/functions/fold.jsonnet".into(),
        replace_string: "x: self.l".into(),
        replace_by_string: "x: self.l.".into(),
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
fn foldr_basic() {
    CompletionTestCase {
        filename: "testdata/complete/std/functions/fold.jsonnet".into(),
        replace_string: "x: self.l".into(),
        replace_by_string: "x: self.r.".into(),
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
#[ignore = "not working"]
fn foldl_object_from_field() {
    CompletionTestCase {
        filename: "testdata/complete/std/functions/fold.jsonnet".into(),
        replace_string: "x: self.l".into(),
        replace_by_string: "x: self.c.".into(),
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
#[ignore = "stack overflow"]
fn foldl_object_from_func_return() {
    CompletionTestCase {
        filename: "testdata/complete/std/functions/fold.jsonnet".into(),
        replace_string: "y: retFunc(concatObject([{ inner: 0 }])).inner".into(),
        replace_by_string: "y: retFunc(concatObject([{ inner: 0 }])).".into(),
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
