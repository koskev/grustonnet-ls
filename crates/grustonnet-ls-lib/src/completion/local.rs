use std::{sync::Arc, time::Instant};

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    completion::std::StdCompletion,
    documentation::DocumentationInfo,
    node::{
        stack::NodeStack,
        types::{
            desugared_object::DesugaredObjectField, function::Apply, node::Node,
            node_kind::NodeKind,
        },
    },
};
use anyhow::Result;
use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
    utils::UriHelper,
};
use lsp_types::{CompletionItem, CompletionItemLabelDetails, CompletionList, Position, Uri};
use thiserror::Error;

pub struct LocalCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> LocalCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("getting index name")]
    IndexName,
    #[error("finding DesugaredObject")]
    NoDesugaredObject,
    #[error("resolving last node of call stack")]
    ReolveLastNode,
    #[error("creating callstack iter")]
    CreateCallstack,
    #[error("no base object found")]
    NoBaseObject,
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

    /// The number of max iterations to avoid endless loops not considered in the code
    pub iterations_left: u32,
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
            iterations_left: 100_000,
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
            let ext_node: Arc<Node> = self
                .cache
                .ast_generator
                .jsonnet
                .get_ast_snippet_binary(&current_node.node_base.loc_range.file_name, val)
                .ok()?
                .into();
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
        let found_object =
            stack_iter.find(|n| matches!(n.node_kind.as_ref(), NodeKind::DesugaredObject(_)))?;

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

    fn handle_node(&mut self, current_node: Arc<Node>) -> Option<Arc<Node>> {
        let start = Instant::now();
        log::info!("{}", current_node.node_kind.variant_name());
        let name = current_node.node_kind.variant_name();
        let result = match current_node.node_kind.as_ref() {
            NodeKind::Other => {
                log::error!("Got invalid node");
                None
            }
            NodeKind::Index(_idx) => {
                let compiled_index: Arc<Node> =
                    CallStackIter::new(self.cache, self.document_stack)?.last()?;
                self.search_stack.push(compiled_index.clone());
                Some(compiled_index)
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

                if let Some(resolved) = var.resolve(self.document_stack) {
                    log::debug!(
                        "{} Resolved to {:?} at {}:{:?}",
                        var.id.clone().unwrap_or_default().0,
                        resolved.node_kind.variant_name(),
                        resolved.node_base.loc_range.file_name,
                        resolved.node_base.loc_range.begin,
                    );
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
                    let jpaths = self.cache.ast_generator.jsonnet.get_evaluate_params(&current_node.node_base.loc_range.file_name).jpaths;
                    let imported_node = jpaths.iter().find_map(|p| self.cache.get_document(
                            &Uri::from_path(&format!("{}/{}", p, file.value)).ok()?).ok()?.ast)?;

                            log::debug!(
                                "pushing import node {} for {}",
                                imported_node.node_kind,
                                file.value,
                            );
                            self.search_stack.push(imported_node.clone());
                            Some(imported_node)
                } else {
                    log::error!("Import file is not a string!");
                    None
                }
            }
            NodeKind::Apply(apply) => {
                // If the target is an index that points to std: we need to handle the current
                // apply node as an std function
                // TODO: $std for loops etc.

                let start_apply = Instant::now();
                if let NodeKind::Index(idx) = apply.target.node_kind.as_ref() {
                    if let NodeKind::Var(var) = idx.target.node_kind.as_ref()
                        && var.is_std()
                    {
                        // Handle the std node
                        // extVar: We can't compile the node due to hidden fields
                        let res =  match idx.get_name().unwrap_or_default().as_str() {
                            "extVar" => self.handle_extvar(&current_node, apply),
                            // TODO: just compile the node
                            _ => None,
                        };
                        log::info!("Apply took for match {:?}", start_apply.elapsed());
                        return res
                    }
                }

                self.search_stack.push(apply.target.clone());
                log::trace!("Got apply {}", apply.target.node_kind);
                // TODO: find function
                // get names of positional arguments and push them to the document stack

                log::info!("Apply took {:?}", start_apply.elapsed());
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
            NodeKind::SuperIndex(_) => self.handle_self_super(&current_node, true),
            NodeKind::SelfNode => self.handle_self_super(&current_node, false),
            NodeKind::Conditional(cond) => {
                let resolved = cond.resolve().clone();
                self.search_stack.push(resolved.clone());
                // TODO: this breaks outer assert. Is it even needed?
                //self.next_nodes.push(cond.cond.clone());
                self.next_nodes.push(cond.branch_false.clone());
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
            // Return the current node without adding it to the search stack to avoid completing
            // the previous index again. e.g. foo.bar.bar.bar
            NodeKind::LiteralString(_)
            | NodeKind::LiteralNumber(_)
            | NodeKind::LiteralBoolean(_)
            | NodeKind::LiteralNull
            | NodeKind::ImportStr(_)
            | NodeKind::ImportBin(_)
            | NodeKind::Array(_) // Handled by the callstack iter
            // TODO: check the the kinds below
            | NodeKind::Error(_)
            | NodeKind::Unary(_)
            | NodeKind::InSuper(_) => Some(current_node),
        };
        log::info!("Handle node took {:?}: {}", start.elapsed(), name);
        result
    }
}

impl<'a> Iterator for ResolveNodeIter<'a> {
    type Item = Arc<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iterations_left == 0 {
            return None;
        }
        self.iterations_left -= 1;
        if let Some(next_node) = self.next_nodes.pop() {
            self.search_stack.push(next_node.clone());
            return Some(next_node);
        }
        while let Some(current_node) = self.search_stack.stack.pop() {
            log::info!("Looking at {}", current_node.node_kind.variant_name());
            self.document_stack.push(current_node.clone());
            let start = Instant::now();
            if let Some(resolved) = self.handle_node(current_node) {
                log::info!("Successfull handled node in {:?}", start.elapsed());
                return Some(resolved);
            }
            log::info!("failed to handle node in {:?}", start.elapsed());
        }
        log::info!(
            "Search stack is empty. Checking if there are nodes to merge. Len {}",
            self.merge_nodes.len()
        );
        let top_node = self.merge_nodes.pop()?;
        let mut merged_node = (*top_node).clone();

        let NodeKind::DesugaredObject(mut merged_object) = merged_node.node_kind.as_ref().clone()
        else {
            return None;
        };
        while let Some(other_node) = self.merge_nodes.pop() {
            if let NodeKind::DesugaredObject(obj) = other_node.node_kind.as_ref() {
                merged_object = merged_object.merge(obj);
            }
        }
        merged_node.node_kind = Box::new(NodeKind::DesugaredObject(merged_object));
        Some(merged_node.into())
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
        log::trace!(
            "New callstack iter with stack\n{}\nfrom\n{}",
            call_stack,
            document_stack
        );
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
        log::trace!("New call node: {}", call_node.node_kind);
        // Get the next object to complete. If we don't have a base object: Just use the call node
        // if we have a base object: Check for the DesugaredObject fields and get the correct one
        let to_complete_object = match &self.base_object {
            None => call_node,
            Some(base_object) => match call_node.node_kind.as_ref() {
                NodeKind::Index(idx) => {
                    match base_object.node_kind.as_ref() {
                        NodeKind::DesugaredObject(obj) => {
                            let index_name = idx.get_name()?;
                            let found_field = obj.get_field(&index_name)?;
                            found_field.body.clone()
                        }
                        // arr[0] is basically arr.0
                        NodeKind::Array(arr) => {
                            if let NodeKind::LiteralNumber(idx_num) = idx.index.node_kind.as_ref()
                                && let Ok(idx_num) = idx_num.original_string.parse::<usize>()
                                && let Some(element) = arr.elements.get(idx_num)
                            {
                                element.expr.clone()
                            } else {
                                base_object.clone()
                            }
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
        document_stack: &mut NodeStack,
    ) -> Result<Arc<Node>> {
        let mut base_object: Option<Arc<Node>> = None;

        while let Some(call_node) = call_stack.stack.pop() {
            let to_complete_object = match base_object {
                None => call_node,
                Some(base_object) => match call_node.node_kind.as_ref() {
                    NodeKind::Index(idx) => {
                        let index_name = idx.get_name().ok_or(LocalError::IndexName)?;
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
                                    .ok_or(LocalError::NoDesugaredObject)?;
                                found_field.body.clone()
                            }
                            _ => base_object,
                        }
                    }
                    _ => base_object,
                },
            };
            base_object = Some(
                ResolveNodeIter::new(to_complete_object, document_stack, self.cache)
                    .last()
                    .ok_or(LocalError::ReolveLastNode)?,
            );
        }
        base_object.ok_or(LocalError::NoBaseObject.into())
    }

    pub fn build_node(&self, document_stack: NodeStack) -> Result<Arc<Node>> {
        let mut document_stack = document_stack;
        let iter = CallStackIter::new(self.cache, &mut document_stack)
            .ok_or(LocalError::CreateCallstack)?;
        iter.last().ok_or(LocalError::ReolveLastNode.into())
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
        let mut last_docsonnet_node: Option<&DesugaredObjectField> = None;
        let items = match node.node_kind.as_ref() {
            NodeKind::DesugaredObject(obj) => obj
                .fields
                .iter()
                .filter_map(|field| {
                    if field.get_name()?.starts_with("#") {
                        last_docsonnet_node = Some(field);
                    }
                    let mut detail = field.body.node_kind.get_value();
                    // TODO: better detection
                    if let Some(documentation_node) = &last_docsonnet_node
                        && documentation_node.get_name().unwrap() == field.get_name().unwrap()
                    {
                        let doc_info = DocumentationInfo::from_docsonnet_node(
                            self.cache,
                            documentation_node.body.clone(),
                        );
                        if let Some(doc_info) = doc_info {
                            detail = Some(doc_info.help_text);
                        }
                    }
                    Some(CompletionItem {
                        label: field.get_name()?,
                        detail,
                        kind: Some(field.body.node_kind.get_lsp_kind()),
                        label_details: Some(CompletionItemLabelDetails {
                            description: Some(field.body.node_kind.get_node_kind_name().into()),
                            ..Default::default()
                        }),

                        ..Default::default()
                    })
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
                log::warn!("Unhandled local completion: {}", node.node_kind);
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
