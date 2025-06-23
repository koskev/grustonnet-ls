use std::fs::read_to_string;

use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};
use pretty_assertions::assert_eq;

use crate::{
    cache::Cache,
    completion::{Completion, global::GlobalCompletion},
};

struct GlobalTestCase {
    filename: String,
    replace_string: String,
    replace_by_string: String,
    expected: CompletionList,
}

impl GlobalTestCase {
    fn check(&self) {
        // Load file
        let file_content = read_to_string(&self.filename).unwrap();
        let string_begin = file_content
            .find(&self.replace_string)
            .expect(format!("Unable to find {} in {}", self.replace_string, file_content).as_str());
        let string_end = string_begin + self.replace_string.len();
        let mut rope = ropey::Rope::from(file_content.clone());
        rope.remove(string_begin..string_end);
        rope.insert(string_begin, &self.replace_by_string);
        let line = rope.char_to_line(string_begin);
        let char = string_begin - rope.line_to_char(line);

        let cache = Cache::default();
        cache.update_content(&self.filename, &file_content);

        let mut completion_list = GlobalCompletion::new(&cache).complete(
            crate::node::location::Location {
                line: line as i32,
                column: char as i32,
            },
            &self.filename,
        );

        for item in completion_list.items.iter_mut() {
            item.detail = None;
        }

        assert_eq!(self.expected, completion_list)
    }
}

macro_rules! global_test {
    ($name:ident, $filename:expr, $replace_string:expr, $replace_by_string:expr, $result_list:expr) => {
        #[test]
        fn $name() {
            GlobalTestCase {
                filename: $filename.to_string(),
                replace_string: $replace_string.to_string(),
                replace_by_string: $replace_by_string.to_string(),
                expected: $result_list,
            }
            .check();
        }
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
