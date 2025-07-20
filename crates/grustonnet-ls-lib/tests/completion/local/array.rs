use super::*;

#[test]
fn simple_index_invalid() {
    CompletionTestCase {
        filename: "testdata/complete/array/index.jsonnet".into(),
        replace_string: "myArr[0]".into(),
        replace_by_string: "myArr[5].".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_index_zero() {
    CompletionTestCase {
        filename: "testdata/complete/array/index.jsonnet".into(),
        replace_string: "myArr[0]".into(),
        replace_by_string: "myArr[0].".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "keyZero".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_index_one() {
    CompletionTestCase {
        filename: "testdata/complete/array/index.jsonnet".into(),
        replace_string: "myArr[0]".into(),
        replace_by_string: "myArr[1].".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "keyOne".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_index_zero_nested() {
    CompletionTestCase {
        filename: "testdata/complete/array/index.jsonnet".into(),
        replace_string: "myArr[0]".into(),
        replace_by_string: "myArr[0].keyZero.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "innerZero".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn simple_index_one_nested() {
    CompletionTestCase {
        filename: "testdata/complete/array/index.jsonnet".into(),
        replace_string: "myArr[0]".into(),
        replace_by_string: "myArr[1].keyOne.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "innerOne".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
