use lsp_types::{CompletionItem, CompletionList};

use crate::{cache::Cache, completion::Completion};

pub struct KeywordCompletion<'a> {
    cache: &'a Cache,
}

impl<'a> KeywordCompletion<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

impl<'a> Completion for KeywordCompletion<'a> {
    fn complete(
        &self,
        location: crate::node::location::Location,
        _: &str,
    ) -> lsp_types::CompletionList {
        let mut items = vec![];

        items.push(CompletionItem {
            label: "local".into(),
            ..Default::default()
        });

        CompletionList {
            items,
            ..Default::default()
        }
    }
}
