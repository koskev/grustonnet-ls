use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, Position, Uri};

use crate::{
    cache::JsonnetASTGenerator,
    node::{stack::NodeStack, types::node_kind::NodeKind},
};

pub struct KeywordCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> KeywordCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

fn show_super(stack: &NodeStack) -> bool {
    // Find the first parent binary
    let first_binary = stack.stack.iter().rev().find_map(|node| {
        if let NodeKind::Binary(bin) = node.node_kind.as_ref() {
            Some(bin)
        } else {
            None
        }
    });
    // If the left one has an object -> super is valid
    if let Some(first_binary) = first_binary {
        match first_binary.left.node_kind.as_ref() {
            NodeKind::DesugaredObject(_) => {
                // If the left one is the same as the current object we only have rights in the
                // binary
                first_binary.left != stack.peek().unwrap_or_default()
            }
            NodeKind::Binary(bin) => {
                let nodes = bin.flatten();
                nodes
                    .iter()
                    .any(|n| matches!(n.node_kind.as_ref(), NodeKind::DesugaredObject(_)))
            }
            _ => false,
        }
    } else {
        false
    }
}

impl<'a> Completion for KeywordCompletion<'a> {
    fn complete(&self, location: Position, uri: &Uri) -> CompletionResult {
        let doc = self.cache.get_document(uri).unwrap();

        let stack = doc.get_ast()?.get_stack_by_position(&location.into());

        let show_self = stack
            .stack
            .iter()
            .any(|node| matches!(*node.node_kind, NodeKind::DesugaredObject(_)));
        // TODO: check if keywords are really usable
        let mut keywords = vec!["local", "import", "importstr"];
        if show_self {
            keywords.push("self");
        }
        if show_super(&stack) {
            keywords.push("super");
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
