pub use crate::cache::Cache;
pub use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};
pub use pretty_assertions::assert_eq;
pub use std::fs::read_to_string;
use std::sync::Once;

use crate::{completion::Completion, utils::rope::RopeHelper};

pub(crate) struct CompletionTestCase<T: Completion> {
    pub(crate) filename: String,
    pub(crate) replace_string: String,
    pub(crate) replace_by_string: String,
    pub(crate) expected: CompletionList,

    pub(crate) provider: T,
}

pub static INIT: Once = Once::new();

impl<T: Completion> CompletionTestCase<T> {
    pub(crate) fn check(&self, cache: &Cache) {
        // Load file
        let file_content = read_to_string(&self.filename).unwrap();
        cache.update_content(&self.filename, &file_content);
        let mut rope = ropey::Rope::from(file_content.clone());
        let location = rope
            .replace_get_end(&self.replace_string, &self.replace_by_string)
            .expect("Unable to replace string");

        cache.update_content(&self.filename, rope.to_string().as_str());

        let mut completion_list = self.provider.complete(location.clone(), &self.filename);

        for item in completion_list.items.iter_mut() {
            item.detail = None;
        }

        assert_eq!(self.expected, completion_list, "At {:?}", location)
    }
}

macro_rules! completion_test {
    ($name:ident, $filename:expr, $replace_string:expr, $replace_by_string:expr, $result_list:expr, $provider:expr) => {
        #[test]
        fn $name() {
            INIT.call_once(|| {
                env_logger::init();
            });
            let cache = Cache::default();
            CompletionTestCase {
                filename: $filename.to_string(),
                replace_string: $replace_string.to_string(),
                replace_by_string: $replace_by_string.to_string(),
                expected: $result_list,
                provider: $provider(&cache),
            }
            .check(&cache);
        }
    };
}
pub(crate) use completion_test;
