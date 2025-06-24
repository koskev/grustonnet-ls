use crate::completion::global::GlobalCompletion;

use crate::completion::test::completion_test::*;

macro_rules! global_test {
    ($name:ident, $filename:expr, $replace_string:expr, $replace_by_string:expr, $result_list:expr) => {
        completion_test!(
            $name,
            $filename,
            $replace_string,
            $replace_by_string,
            $result_list,
            GlobalCompletion::new
        );
    };
}

global_test!(
    simple_local,
    "testdata/simple_local.jsonnet",
    "x: myVar,",
    "x: my",
    CompletionList {
        is_incomplete: false,
        items: vec![CompletionItem {
            label: "myVar".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            ..Default::default()
        }],
    }
);
global_test!(
    simple_local_func,
    "testdata/simple_local_func.jsonnet",
    "x: myFunc(),",
    "x: my",
    CompletionList {
        is_incomplete: false,
        items: vec![CompletionItem {
            label: "myFunc".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            ..Default::default()
        }],
    }
);
