use anyhow::{Result, anyhow};
use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionList, Position};

use crate::{
    cache::JsonnetASTGenerator,
    completion::std::StdCompletion,
    node::{Node, NodeKind, stack::NodeStack},
};

pub struct LocalCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> LocalCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

pub struct ResolveNodeIter<'a> {
    pub search_stack: NodeStack,

    // The complete document stack. Used to search for variables etc.
    // Every node that lands on the search stack also lands here
    pub document_stack: &'a mut NodeStack,
    pub cache: &'a Cache<JsonnetASTGenerator>,

    pub binary_index: i32,
}

impl<'a> ResolveNodeIter<'a> {
    pub fn new(
        node: Node,
        document_stack: &'a mut NodeStack,
        cache: &'a Cache<JsonnetASTGenerator>,
    ) -> Self {
        let mut search_stack = NodeStack::new();
        search_stack.push(node);
        Self {
            search_stack,
            document_stack,
            cache,
            binary_index: 0,
        }
    }
}

impl<'a> Iterator for ResolveNodeIter<'a> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        let current_node = self.search_stack.stack.pop()?;
        log::debug!("Looking at {}", current_node.node_kind.variant_name());
        self.document_stack.push(current_node.clone());
        match &(*current_node.node_kind) {
            NodeKind::Other(other) => {
                log::error!("Got invalid node {:#?}", other);
                None
            }
            NodeKind::Index(idx) => {
                self.search_stack.push(idx.target.clone());
                Some(idx.target.clone())
            }
            NodeKind::DesugaredObject(_obj) => {
                log::error!("Found desugared!");
                Some(current_node)
            }
            NodeKind::Var(var) => {
                // TODO: For now we'll just return. In the future we need to evaluate the call
                if var.is_std() {
                    return Some(current_node);
                }

                if let Some(resolved) = var.resolve(&self.document_stack) {
                    log::warn!("Resolved to {:?}", resolved.node_kind.variant_name());
                    self.search_stack.push(resolved.clone());
                    Some(resolved.clone())
                } else {
                    log::warn!(
                        "Unable to resolve var {}",
                        var.id.clone().unwrap_or_default().0
                    );
                    None
                }
            }
            NodeKind::Local(local) => {
                if let Some(body) = &local.body {
                    self.search_stack.push(body.clone());
                    Some(body.clone())
                } else {
                    None
                }
            }
            NodeKind::Import(import) => {
                if let NodeKind::LiteralString(file) = import.file.node_kind.as_ref() {
                    let imported = self
                        .cache
                        .ast_generator
                        .import_ast(&current_node.node_base.loc_range.file_name, &file.value);
                    match imported {
                        Ok(imported_node) => {
                            log::error!(
                                "pushing import node {}",
                                imported_node.node_kind.variant_name()
                            );
                            self.search_stack.push(imported_node.clone());
                            Some(imported_node.clone())
                        }
                        Err(e) => {
                            log::error!("Failed to import node: {}", e);
                            None
                        }
                    }
                } else {
                    log::error!("Import file is not a string!");
                    None
                }
            }
            NodeKind::Apply(apply) => {
                self.search_stack.push(apply.target.clone());
                log::debug!("Got apply {}", apply.target.node_kind);
                // TODO: find function
                // get names of positional arguments and push them to the document stack

                Some(apply.target.clone())
            }
            NodeKind::Function(func) => {
                let apply_node = self.document_stack.stack.iter().find_map(|n| {
                    if let NodeKind::Apply(apply) = n.node_kind.as_ref() {
                        Some(apply)
                    } else {
                        None
                    }
                })?;
                // Match arguments from apply to function and push them to the search and
                // document stack
                if let Some(bindings) = func.get_bind_for_arguments(&apply_node.arguments) {
                    log::debug!("Found correct bindings");
                    for binding in bindings {
                        log::error!("Pushing to document stack {:?}", binding);
                        self.document_stack.push(binding);
                    }
                    //document_stack.stack.extend(bindings);
                } else {
                    log::debug!("Failed to find bindings");
                }
                // Push the function body to the stack
                self.search_stack.push(func.body.clone());
                log::debug!("Got function");

                Some(func.body.clone())
            }
            NodeKind::Binary(binary) => {
                if let NodeKind::DesugaredObject(a_desugared) = binary.left.node_kind.as_ref()
                    && let NodeKind::DesugaredObject(b_desugared) = binary.right.node_kind.as_ref()
                {
                    // FIXME: why does this need to be switched?
                    let merged = b_desugared.merge(a_desugared.clone());
                    let mut merged_node = current_node.clone();
                    *merged_node.node_kind = NodeKind::DesugaredObject(merged);
                    Some(merged_node)
                } else {
                    self.binary_index += 1;
                    match self.binary_index {
                        1 => {
                            // Push it back to the search stack to get the right node in the next
                            // iteration
                            self.search_stack.push_front(current_node.clone());
                            self.search_stack.push(binary.left.clone());
                            Some(binary.left.clone())
                        }
                        2 => {
                            self.search_stack.push(binary.right.clone());
                            Some(binary.right.clone())
                        }
                        _ => None,
                    }
                }
            }
            NodeKind::SelfNode => {
                // We need to find the node in the stack. Otherwise, if we have a var, we might reference the
                // current object instead of the var object
                let self_stack = self
                    .document_stack
                    .generate_stack_for_node(current_node.clone());
                log::error!("SELF STACK: {}", self_stack);
                let found_object = self_stack.stack.into_iter().rfind(|n| {
                    if let NodeKind::DesugaredObject(_) = n.node_kind.as_ref() {
                        true
                    } else {
                        false
                    }
                })?;
                self.search_stack.push(found_object.clone());
                Some(found_object.clone())
            }
            _ => {
                log::warn!(
                    "Unhandled node in completion iterator: {}",
                    current_node.node_kind.variant_name(),
                );
                None
            }
        }
    }
}

impl<'a> LocalCompletion<'a> {
    // TODO: make a completion iterator
    pub fn build_node(&self, document_stack: NodeStack) -> Result<Node> {
        let mut call_stack = document_stack
            .peek()
            .ok_or(anyhow!("Could not peek the document stack. Is it empty?"))?
            .get_call_stack();
        log::debug!("Call stack {}", call_stack);
        let mut document_stack = document_stack;

        let base_node = call_stack
            .stack
            .pop()
            .ok_or(anyhow!("Could not pop call stack"))?;
        // TODO: do we need to update the document stack?
        // Pass as mut or other solution?
        let mut base_object =
            ResolveNodeIter::new(base_node.clone(), &mut document_stack, self.cache)
                .filter(|n| matches!(*n.node_kind, NodeKind::DesugaredObject(_)))
                // There might be other desugared objects e.g. in locals. We a interested in the
                // last one
                .last()
                .ok_or(anyhow!("Node is not an object"))?;

        while let Some(call_node) = call_stack.stack.pop() {
            match *call_node.node_kind {
                NodeKind::Index(idx) => {
                    let index_name = idx.get_name().ok_or(anyhow!("getting index name"))?;
                    match &(*base_object.node_kind) {
                        NodeKind::DesugaredObject(obj) => {
                            let found_field = obj
                                .fields
                                .iter()
                                .find(|field| {
                                    if let Some(field_name) = field.get_name() {
                                        field_name == index_name
                                    } else {
                                        false
                                    }
                                })
                                .ok_or(anyhow!("finding desugared field"))?;
                            base_object = ResolveNodeIter::new(
                                found_field.body.clone(),
                                &mut document_stack,
                                self.cache,
                            )
                            .find(|n| matches!(*n.node_kind, NodeKind::DesugaredObject(_)))
                            .ok_or(anyhow!("getting new base object"))?;
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        Ok(base_object)
    }
}

impl<'a> Completion for LocalCompletion<'a> {
    fn complete(&self, location: Position, filename: &str) -> CompletionResult {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.get_ast()?.get_stack_by_position(&location.into());
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
        let node = self.build_node(stack)?;
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
