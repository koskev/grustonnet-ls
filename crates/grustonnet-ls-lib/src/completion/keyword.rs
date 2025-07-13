use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, Position, Uri};

use crate::{cache::JsonnetASTGenerator, node::types::node_kind::NodeKind};

pub struct KeywordCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> KeywordCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> Completion for KeywordCompletion<'a> {
    fn complete(&self, location: Position, uri: &Uri) -> CompletionResult {
        let doc = self.cache.get_document(uri).unwrap();

        let stack = doc.get_ast()?.get_stack_by_position(&location.into());

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

        Ok(CompletionList {
            items,
            ..Default::default()
        })
    }
}
