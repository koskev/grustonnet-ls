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

local_test!(
    simple_local,
    "testdata/simple_object.jsonnet",
    "x: object,",
    "x: object.k",
    CompletionList {
        is_incomplete: false,
        items: vec![CompletionItem {
            label: "key".to_string(),
            ..Default::default()
        }],
    }
);
