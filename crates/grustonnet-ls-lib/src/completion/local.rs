use std::{sync::Arc, time::Instant};

use anyhow::{Result, anyhow};
use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionList, Position, Uri};

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    completion::std::StdCompletion,
    node::{
        stack::NodeStack,
        types::{function::Apply, node::Node, node_kind::NodeKind},
    },
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

    // TODO: Use a proper solution inside the binary case. Maybe a recursive Iterator?
    /// DesugaredObject to merge (should all be from a binary)
    merge_nodes: Vec<Arc<Node>>,

    /// Nodes to search with priority (used if a node returns multiple nodes. e.g. a binary)
    next_nodes: Vec<Arc<Node>>,
}

impl<'a> ResolveNodeIter<'a> {
    pub fn new(
        node: Arc<Node>,
        document_stack: &'a mut NodeStack,
        cache: &'a Cache<JsonnetASTGenerator>,
    ) -> Self {
        let mut search_stack = NodeStack::new();
        search_stack.push(node);
        Self {
            search_stack,
            document_stack,
            cache,
            merge_nodes: vec![],
            next_nodes: vec![],
        }
    }
}

impl<'a> ResolveNodeIter<'a> {
    fn handle_extvar(&mut self, current_node: &Node, apply: &Apply) -> Option<Arc<Node>> {
        let conf = self.cache.ast_generator.jsonnet.get_config();
        let arg_node = apply.arguments.get_argument(0)?;
        if let NodeKind::LiteralString(name_node) = arg_node.node_kind.as_ref() {
            let val = conf.ext_code.get(&name_node.value)?;
            // Get ast snippet and add to stack
            let ext_ast = self
                .cache
                .ast_generator
                .jsonnet
                .get_ast_snippet(&current_node.node_base.loc_range.file_name, val)
                .ok()?;
            let ext_node = Arc::new(serde_json::from_str::<Node>(&ext_ast).ok()?);
            self.search_stack.push(ext_node.clone());
            Some(ext_node)
        } else {
            None
        }
    }
}

impl<'a> ResolveNodeIter<'a> {
    fn handle_self_super(&mut self, current_node: &Node, is_super: bool) -> Option<Arc<Node>> {
        // We need to find the node in the stack. Otherwise, if we have a var, we might reference the
        // current object instead of the var object

        let mut stack_iter = self.document_stack.stack.iter().rev();
        // Find the object the self node belongs to
        let found_object = stack_iter.find(|n| {
            if let NodeKind::DesugaredObject(_) = n.node_kind.as_ref() {
                true
            } else {
                false
            }
        })?;

        // The next node is a binary
        // TODO: Check with weird examples if this is correct and we won't find an
        // unrelated binary or not the one we are searching for
        if let Some(next_item) = stack_iter.next()
            && matches!(*next_item.node_kind, NodeKind::Binary(_))
        {
            // Find all binaries in a row and keep the top one
            let binary_pos = if is_super {
                self.document_stack
                    .stack
                    .iter()
                    .rposition(|n| matches!(*n.node_kind, NodeKind::Binary(_)))
            } else {
                self.document_stack
                    .stack
                    .iter()
                    .position(|n| matches!(*n.node_kind, NodeKind::Binary(_)))
            }
            .unwrap_or(0);
            let NodeKind::Binary(binary) = self.document_stack.stack[binary_pos].node_kind.as_ref()
            else {
                log::error!("BUG: Binary is not there");
                return None;
            };
            let mut nodes: Vec<Arc<Node>> = binary
                .flatten()
                .iter()
                // Filter out self to avoid an endless loop
                .filter(|n| ***n != *current_node)
                .map(|n| (*n).clone())
                .rev()
                .collect();
            let first_node = nodes.pop()?;
            self.search_stack.push(first_node.clone());
            if let Some(node) = nodes.pop() {
                self.next_nodes.append(&mut nodes);
                self.search_stack.push(node);
            }
            return Some(first_node);
        }
        self.search_stack.push(found_object.clone());
        Some(found_object.clone())
    }
}

impl<'a> Iterator for ResolveNodeIter<'a> {
    type Item = Arc<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.next_nodes.pop() {
            self.search_stack.push(next_node.clone());
            return Some(next_node);
        }
        let Some(current_node) = self.search_stack.stack.pop() else {
            log::debug!(
                "Search stack is empty. Checking if there are nodes to merge. Len {}",
                self.merge_nodes.len()
            );
            let top_node = self.merge_nodes.pop()?;
            let mut merged_node = (*top_node).clone();

            let NodeKind::DesugaredObject(mut merged_object) =
                merged_node.node_kind.as_ref().clone()
            else {
                return None;
            };
            while let Some(other_node) = self.merge_nodes.pop() {
                if let NodeKind::DesugaredObject(obj) = other_node.node_kind.as_ref() {
                    merged_object = merged_object.merge(obj);
                }
            }
            merged_node.node_kind = Box::new(NodeKind::DesugaredObject(merged_object));
            return Some(merged_node.into());
        };
        log::trace!("Looking at {}", current_node.node_kind);
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
                log::debug!("Found desugared! {}", current_node.node_kind);
                self.merge_nodes.push(current_node.clone());
                Some(current_node)
            }
            NodeKind::Var(var) => {
                // TODO: For now we'll just return. In the future we need to evaluate the call
                if var.is_std() {
                    return Some(current_node);
                }
                if var.is_dollar() {
                    let dollar_node = Arc::new(Node {
                        node_base: current_node.node_base.clone(),
                        node_kind: Box::new(NodeKind::Dollar),
                    });
                    self.search_stack.push(dollar_node.clone());
                    return Some(dollar_node);
                }

                if let Some(resolved) = var.resolve(&mut self.document_stack) {
                    log::debug!("Resolved to {:?}", resolved.node_kind.variant_name());
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
                            let imported_node = Arc::new(imported_node);
                            log::debug!(
                                "pushing import node {} for {}",
                                imported_node.node_kind,
                                file.value,
                            );
                            self.search_stack.push(imported_node.clone());
                            Some(imported_node)
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
                // If the target is an index that points to std: we need to handle the current
                // apply node as an std function
                // TODO: $std for loops etc.

                if let NodeKind::Index(idx) = apply.target.node_kind.as_ref() {
                    if let NodeKind::Var(var) = idx.target.node_kind.as_ref()
                        && var.is_std()
                    {
                        // Handle the std node
                        // extVar: We can't compile the node due to hidden fields
                        return match idx.get_name().unwrap_or_default().as_str() {
                            "extVar" => self.handle_extvar(&current_node, apply),
                            // TODO: just compile the node
                            _ => None,
                        };
                    }
                }

                self.search_stack.push(apply.target.clone());
                log::trace!("Got apply {}", apply.target.node_kind);
                // TODO: find function
                // get names of positional arguments and push them to the document stack

                Some(apply.target.clone())
            }
            NodeKind::Function(func) => {
                log::trace!("Got function. Stack: {}", self.document_stack);
                if let Some(apply_node) = self.document_stack.stack.iter().find_map(|n| {
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
                            self.document_stack.push(binding.into());
                        }
                        //document_stack.stack.extend(bindings);
                    } else {
                        log::debug!("Failed to find bindings");
                    }
                }
                // Push the function body to the stack
                self.search_stack.push(func.body.clone());

                Some(func.body.clone())
            }
            NodeKind::Binary(binary) => {
                self.next_nodes.push(binary.left.clone());
                self.search_stack.push(binary.right.clone());
                Some(binary.right.clone())
            }
            NodeKind::SuperIndex => self.handle_self_super(&current_node, true),
            NodeKind::SelfNode => self.handle_self_super(&current_node, false),
            NodeKind::Conditional(cond) => {
                let resolved = cond.resolve().clone();
                self.search_stack.push(resolved.clone());
                Some(resolved)
            }
            NodeKind::Dollar => {
                // Get the outer most object
                if let Some(first_node) = self
                    .document_stack
                    .find_last_and_skip(|n| matches!(n, NodeKind::DesugaredObject(_)))
                {
                    // TODO: support for binary
                    match first_node.node_kind.as_ref() {
                        NodeKind::DesugaredObject(_obj) => Some(first_node.clone()),
                        NodeKind::Binary(_binary) => None,
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => {
                log::warn!(
                    "Unhandled node in completion iterator: {}",
                    current_node.node_kind.variant_name(),
                );
                // Return the current node as the result to avoid getting the result for the
                // previous index e.g. foo.bar.bar.bar
                Some(current_node)
            }
        }
    }
}

pub struct CallStackIter<'a> {
    pub call_stack: NodeStack,
    pub base_object: Option<Arc<Node>>,

    pub document_stack: &'a mut NodeStack,
    pub cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> CallStackIter<'a> {
    pub fn new(
        cache: &'a Cache<JsonnetASTGenerator>,
        document_stack: &'a mut NodeStack,
    ) -> Option<Self> {
        let call_stack = document_stack.peek()?.get_call_stack();
        log::trace!("New callstack iter with stack {}", call_stack);
        Some(Self {
            cache,
            base_object: None,
            document_stack,
            call_stack,
        })
    }

    pub fn new_with_call_stack(
        cache: &'a Cache<JsonnetASTGenerator>,
        document_stack: &'a mut NodeStack,
        call_stack: NodeStack,
    ) -> Option<Self> {
        Some(Self {
            cache,
            base_object: None,
            document_stack,
            call_stack,
        })
    }
}

// This iterator resolves one of a.b.c.d in every iteration
impl<'a> Iterator for CallStackIter<'a> {
    type Item = Arc<Node>;
    fn next(&mut self) -> Option<Self::Item> {
        let call_node = self.call_stack.stack.pop()?;
        // Get the next object to complete. If we don't have a base object: Just use the call node
        // if we have a base object: Check for the DesugaredObject fields and get the correct one
        let to_complete_object = match &self.base_object {
            None => call_node,
            Some(base_object) => match call_node.node_kind.as_ref() {
                NodeKind::Index(idx) => {
                    let index_name = idx.get_name()?;
                    match base_object.node_kind.as_ref() {
                        NodeKind::DesugaredObject(obj) => {
                            let found_field = obj.get_field(&index_name)?;
                            found_field.body.clone()
                        }
                        // Index does not point to an object
                        _ => base_object.clone(),
                    }
                }
                // Not an index
                _ => base_object.clone(),
            },
        };
        // Actually resolve the object
        let new_object =
            ResolveNodeIter::new(to_complete_object, self.document_stack, self.cache).last()?;
        log::trace!(
            "New object: {} Stack: {}",
            new_object.node_kind,
            self.document_stack
        );
        self.base_object = Some(new_object);
        self.base_object.clone()
    }
}

impl<'a> LocalCompletion<'a> {
    pub fn build_node_from_call_stack(
        &self,
        mut call_stack: NodeStack,
        mut document_stack: &mut NodeStack,
    ) -> Result<Arc<Node>> {
        let mut base_object: Option<Arc<Node>> = None;

        while let Some(call_node) = call_stack.stack.pop() {
            let to_complete_object = match base_object {
                None => call_node,
                Some(base_object) => match call_node.node_kind.as_ref() {
                    NodeKind::Index(idx) => {
                        let index_name = idx.get_name().ok_or(anyhow!("getting index name"))?;
                        match base_object.node_kind.as_ref() {
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
                                found_field.body.clone()
                            }
                            _ => base_object,
                        }
                    }
                    _ => base_object,
                },
            };
            base_object = Some(
                ResolveNodeIter::new(to_complete_object, &mut document_stack, self.cache)
                    .last()
                    .ok_or(anyhow!("getting new base object"))?,
            );
        }
        Ok(base_object.ok_or(anyhow!("no object found"))?)
    }

    pub fn build_node(&self, document_stack: NodeStack) -> Result<Arc<Node>> {
        let mut document_stack = document_stack;
        let iter = CallStackIter::new(self.cache, &mut document_stack)
            .ok_or(anyhow!("Could not create callstack iter"))?;
        iter.last()
            .ok_or(anyhow!("Could not resolve last node of call stack"))
    }
}

impl<'a> Completion for LocalCompletion<'a> {
    fn complete(&self, location: Position, uri: &Uri) -> CompletionResult {
        let start = Instant::now();
        let doc = self.cache.get_document(uri).unwrap();

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
        log::trace!("Built node {}", node.node_kind);
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
                    StdCompletion::new().complete(location, uri)?.items
                } else {
                    log::warn!("Tried to complete var that is not std! {}", node.node_kind);
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

        let dur = start.elapsed();
        log::info!("Local completion took {:?}", dur);

        Ok(CompletionList {
            items,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {}
