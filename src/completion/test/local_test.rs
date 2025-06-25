use crate::completion::local::LocalCompletion;

use crate::completion::test::completion_test::*;

macro_rules! local_test {
    ($name:ident, $filename:expr, $replace_string:expr, $replace_by_string:expr, $result_list:expr) => {
        completion_test!(
            $name,
            $filename,
            $replace_string,
            $replace_by_string,
            $result_list,
            LocalCompletion::new
        );
    };
}

#[test]
#[ignore = "Currently not supported"]
fn simple_local() {
    let cache = Cache::default();
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
        provider: LocalCompletion::new(&cache),
    }
    .check(&cache);
}

#[test]
fn simple_local_no_text() {
    let cache = Cache::default();
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
        provider: LocalCompletion::new(&cache),
    }
    .check(&cache);
}

#[test]
fn object_multiple_no_text() {
    let cache = Cache::default();
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
        provider: LocalCompletion::new(&cache),
    }
    .check(&cache);
}

#[test]
#[ignore = "Currently not supported"]
fn object_multiple() {
    let cache = Cache::default();
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
        provider: LocalCompletion::new(&cache),
    }
    .check(&cache);
}

#[test]
fn object_nested() {
    let cache = Cache::default();
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
        provider: LocalCompletion::new(&cache),
    }
    .check(&cache);
}
