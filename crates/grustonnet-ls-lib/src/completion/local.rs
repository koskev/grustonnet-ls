use anyhow::anyhow;
use language_server::cache::Cache;
use lsp_types::{CompletionItem, CompletionList};

use crate::{
    cache::JsonnetASTGenerator,
    completion::{Completion, CompletionResult, std::StdCompletion},
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
    fn get_desugared_object(&self, node: Node, document_stack: &mut NodeStack) -> Option<Node> {
        let mut search_stack = NodeStack::new();
        let mut objects = vec![];
        search_stack.push(node);
        while let Some(current_node) = search_stack.stack.pop() {
            log::debug!("Looking at {}", current_node.node_kind.variant_name());
            document_stack.push(current_node.clone());
            match &(*current_node.node_kind) {
                NodeKind::Other(other) => {
                    log::error!("Got invalid node {:#?}", other);
                }
                NodeKind::Index(idx) => search_stack.push(idx.target.clone()),
                NodeKind::DesugaredObject(_obj) => {
                    log::error!("Found desugared!");
                    objects.push(current_node);
                }
                NodeKind::Var(var) => {
                    // TODO: For now we'll just return. In the future we need to evaluate the call
                    if var.is_std() {
                        return Some(current_node);
                    }

                    if let Some(resolved) = var.resolve(&document_stack) {
                        log::warn!("Resolved to {:?}", resolved.node_kind.variant_name());
                        search_stack.push(resolved);
                    } else {
                        log::warn!(
                            "Unable to resolve var {}",
                            var.id.clone().unwrap_or_default().0
                        );
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
                            .import_ast(&current_node.node_base.loc_range.file_name, &file.value);
                        match imported {
                            Ok(imported_node) => search_stack.push(imported_node),
                            Err(e) => log::error!("Failed to import node: {}", e),
                        };
                    }
                }
                NodeKind::Apply(apply) => {
                    search_stack.push(apply.target.clone());
                    log::debug!("Got apply");
                    // TODO: find function
                    // get names of positional arguments and push them to the document stack
                }
                NodeKind::Function(func) => {
                    if let Some(apply_node) = document_stack.stack.iter().find_map(|n| {
                        if let NodeKind::Apply(apply) = n.node_kind.as_ref() {
                            Some(apply)
                        } else {
                            None
                        }
                    }) {
                        // Match arguments from apply to function and push them to the search and
                        // document stack
                        if let Some(bindings) = func.get_bind_for_arguments(&apply_node.arguments) {
                            log::debug!("Found correct bindings");
                            for binding in bindings {
                                log::error!("Pushing to document stack {:?}", binding);
                                document_stack.push(binding);
                            }
                            //document_stack.stack.extend(bindings);
                        } else {
                            log::debug!("Failed to find bindings");
                        }
                        // Push the function body to the stack
                        search_stack.push(func.body.clone());
                    }
                    log::debug!("Got function");
                    // TODO: search the stack for the corresponding apply node
                }
                NodeKind::Binary(binary) => {
                    search_stack.push(binary.left.clone());
                    search_stack.push(binary.right.clone());
                }
                _ => log::warn!(
                    "Unhandled node in completion iterator: {}",
                    current_node.node_kind.variant_name()
                ),
            }
        }
        objects.into_iter().reduce(|a, b| {
            let NodeKind::DesugaredObject(a_desugared) = a.node_kind.as_ref() else {
                // Just hope the other one is valid :)
                return b.clone();
            };
            let NodeKind::DesugaredObject(b_desugared) = b.node_kind.as_ref() else {
                return a.clone();
            };
            let merged = a_desugared.merge(b_desugared.clone());
            let mut merged_node = a.clone();
            *merged_node.node_kind = NodeKind::DesugaredObject(merged);
            merged_node
        })
    }

    // TODO: make a completion iterator
    fn build_node(&self, document_stack: NodeStack) -> Option<Node> {
        let mut call_stack = document_stack.peek()?.get_call_stack();
        let mut document_stack = document_stack;

        let mut base_object =
            self.get_desugared_object(call_stack.stack.pop()?, &mut document_stack)?;

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
                                &mut document_stack,
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
    fn complete(&self, location: Location, filename: &str) -> CompletionResult {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.get_ast()?.get_stack_by_position(&location);
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
            return Err(anyhow!("Could not build_node"));
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
                    StdCompletion::new().complete(location, filename)?.items
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

        Ok(CompletionList {
            items,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {}
