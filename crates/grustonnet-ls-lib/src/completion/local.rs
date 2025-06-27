use language_server::cache::Cache;
use lsp_types::{CompletionItem, CompletionList};

use crate::{
    cache::JsonnetASTGenerator,
    completion::{Completion, std::StdCompletion},
    node::{Node, NodeKind, location::Location, stack::NodeStack},
};

pub struct LocalCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> LocalCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> LocalCompletion<'a> {
    fn get_desugared_object(&self, node: Node, document_stack: NodeStack) -> Option<Node> {
        let mut search_stack = NodeStack::new();
        search_stack.push(node);
        while let Some(current_node) = search_stack.stack.pop() {
            log::debug!("Looking at {}", current_node.node_kind.variant_name());
            match &(*current_node.node_kind) {
                NodeKind::Other(other) => {
                    log::error!("Got invalid node {:#?}", other);
                }
                NodeKind::Index(idx) => search_stack.push(idx.target.clone()),
                NodeKind::DesugaredObject(_obj) => {
                    log::error!("Found desugared!");
                    return Some(current_node);
                }
                NodeKind::Var(var) => {
                    // TODO: For now we'll just return. In the future we need to evaluate the call
                    if var.is_std() {
                        return Some(current_node);
                    }

                    if let Some(resolved) = var.resolve(&document_stack) {
                        log::warn!("Resolved to {:?}", resolved.node_kind.variant_name());
                        search_stack.push(resolved);
                    }
                }
                NodeKind::Local(local) => {
                    if let Some(body) = &local.body {
                        search_stack.push(body.clone());
                    }
                }
                NodeKind::Import(import) => {
                    if let NodeKind::LiteralString(file) = import.file.node_kind.as_ref() {
                        let imported = self
                            .cache
                            .ast_generator
                            .import_ast(&current_node.loc_range.file_name, &file.value);
                        match imported {
                            Ok(imported_node) => search_stack.push(imported_node),
                            Err(e) => log::error!("Failed to import node: {}", e),
                        };
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

    // TODO: make a completion iterator
    fn build_node(&self, document_stack: NodeStack) -> Option<Node> {
        let mut call_stack = document_stack.peek()?.get_call_stack();

        let mut base_object =
            self.get_desugared_object(call_stack.stack.pop()?, document_stack.clone())?;

        while let Some(call_node) = call_stack.stack.pop() {
            match *call_node.node_kind {
                NodeKind::Index(idx) => {
                    let index_name = idx.get_name()?;
                    match &(*base_object.node_kind) {
                        NodeKind::DesugaredObject(obj) => {
                            let found_field = obj.fields.iter().find(|field| {
                                if let Some(field_name) = field.get_name() {
                                    field_name == index_name
                                } else {
                                    false
                                }
                            })?;
                            base_object = self.get_desugared_object(
                                found_field.body.clone(),
                                document_stack.clone(),
                            )?;
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        Some(base_object)
    }
}

impl<'a> Completion for LocalCompletion<'a> {
    fn complete(&self, location: Location, filename: &str) -> CompletionList {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.ast.unwrap().get_stack_by_position(&location);
        let top_node = stack.peek().unwrap();
        log::debug!(
            "Completing {} at {:?}",
            top_node.node_kind.variant_name(),
            location
        );
        // TODO: get the current index and use it as the filter for the rest of the completion
        // TODO: Create call stack and get every stage for the completion. Get the first object and
        // use the second one as a filter
        // TODO: Resolve the complete call stack
        let Some(node) = self.build_node(stack) else {
            return CompletionList::default();
        };
        let items = match node.node_kind.as_ref() {
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
            NodeKind::Var(var) => {
                if var.is_std() {
                    StdCompletion::new().complete(location, filename).items
                } else {
                    vec![]
                }
            }
            _ => {
                log::warn!(
                    "Unhandled local completion: {}",
                    node.node_kind.variant_name()
                );
                vec![]
            }
        };

        CompletionList {
            items,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {}
