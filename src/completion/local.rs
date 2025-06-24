use lsp_types::{CompletionItem, CompletionList};

use crate::{
    cache::Cache,
    completion::Completion,
    cst::node_type::NodeType,
    node::{DesugaredObject, Node, NodeKind, location::Location, stack::NodeStack},
};

pub struct LocalCompletion<'a> {
    cache: &'a Cache,
}

impl<'a> LocalCompletion<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

pub struct CompletionIterator {
    search_stack: NodeStack,
    document_stack: NodeStack,
}

impl Iterator for CompletionIterator {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_node) = self.search_stack.stack.pop() {
            match &(*current_node.node_kind) {
                NodeKind::Index(idx) => self.search_stack.push(idx.target.clone()),
                NodeKind::DesugaredObject(_obj) => {
                    log::error!("Found desugared!");
                    return Some(current_node);
                }
                NodeKind::Var(var) => {
                    if let Some(resolved) = var.resolve(&self.document_stack) {
                        log::warn!("Resolved to {:?}", resolved);
                        self.search_stack.push(resolved);
                    }
                }
                NodeKind::Local(local) => {
                    if let Some(body) = &local.body {
                        self.search_stack.push(body.clone());
                    }
                }
                _ => log::warn!(
                    "Unhandled node in completion iterator: {}",
                    current_node.node_kind.variant_name()
                ),
            }
        }

        None
    }
}

impl CompletionIterator {
    fn from_node(node: Node, document_stack: NodeStack) -> Self {
        Self {
            search_stack: node.get_call_stack(),
            document_stack,
        }
    }
}

impl<'a> Completion for LocalCompletion<'a> {
    fn complete(&self, location: Location, filename: &str) -> CompletionList {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.ast.get_stack_by_position(&location);
        let top_node = stack.peek().unwrap();
        log::debug!(
            "Completing {} at {:?}",
            top_node.node_kind.variant_name(),
            location
        );
        let iter = CompletionIterator::from_node(top_node, stack);
        let items = iter
            .flat_map(|node| match *node.node_kind {
                NodeKind::DesugaredObject(obj) => obj
                    .fields
                    .iter()
                    .filter_map(|field| match &(*field.name.node_kind) {
                        NodeKind::LiteralString(name) => Some(CompletionItem {
                            label: name.value.clone(),
                            ..Default::default()
                        }),
                        _ => None,
                    })
                    .collect(),
                _ => {
                    log::warn!(
                        "Unhandled local completion: {}",
                        node.node_kind.variant_name()
                    );
                    vec![]
                }
            })
            .collect();

        CompletionList {
            items,
            ..Default::default()
        }
    }
}
