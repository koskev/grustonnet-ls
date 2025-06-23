use std::sync::Arc;

use lsp_types::{CompletionItem, CompletionItemKind};

use crate::{
    cache::Cache,
    completion::Completion,
    node::{NodeKind, location::Location},
};

pub struct GlobalCompletion<'a> {
    cache: &'a Cache,
}

impl<'a> GlobalCompletion<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

impl<'a> Completion for GlobalCompletion<'a> {
    fn complete(&self, pos: Location, filename: &str) -> lsp_types::CompletionList {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.ast.get_stack_by_position(&pos);
        let items: Vec<CompletionItem> = stack
            .stack
            .iter()
            .filter_map(|node| match &(*node.node_kind) {
                crate::node::NodeKind::LocalBind(bind) => {
                    eprintln!("Got bind!");
                    Some(CompletionItem {
                        label: bind.variable.clone(),
                        ..Default::default()
                    })
                }
                NodeKind::Local { binds, body } => {
                    eprintln!("Got local!");

                    Some(CompletionItem {
                        label: binds[0].variable.clone(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: match body {
                            Some(body) => Some(body.node_kind.variant_name().to_string()),
                            None => None,
                        },
                        ..Default::default()
                    })
                }
                _ => {
                    eprintln!("No bind {}", node.node_kind.variant_name());
                    None
                }
            })
            .collect();
        lsp_types::CompletionList {
            items,
            is_incomplete: false,
        }
    }
}
