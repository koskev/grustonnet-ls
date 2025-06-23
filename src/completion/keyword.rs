use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};

use crate::{cache::Cache, completion::Completion, node::NodeKind};

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
        filename: &str,
    ) -> lsp_types::CompletionList {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.ast.get_stack_by_position(&location);

        let show_self = stack.stack.iter().any(|node| {
            if let NodeKind::DesugaredObject(_) = *node.node_kind {
                true
            } else {
                false
            }
        });
        // TODO: check if keywords are really usable
        let mut keywords = vec!["local", "import", "importstr", "super"];
        if show_self {
            keywords.push("self");
        }

        let items = keywords
            .iter()
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            })
            .collect();

        CompletionList {
            items,
            ..Default::default()
        }
    }
}
