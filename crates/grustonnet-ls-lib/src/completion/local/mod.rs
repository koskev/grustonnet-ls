use std::{sync::Arc, time::Instant};

use crate::{
    cache::JsonnetASTGenerator,
    completion::{
        local::{call_stack_iter::CallStackIter, resolve_node_iter::ResolveNodeIter},
        std::StdCompletion,
    },
    documentation::DocumentationInfo,
    node::{
        stack::NodeStack,
        types::{desugared_object::DesugaredObjectField, node::Node, node_kind::NodeKind},
    },
};
use anyhow::Result;
use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionItemLabelDetails, CompletionList, Position, Uri};
use thiserror::Error;

pub mod call_stack_iter;
pub mod resolve_node_iter;

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
        log::trace!("Stack {}", stack);
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
                        && documentation_node.get_name().unwrap()
                            == format!("#{}", field.get_name().unwrap())
                    {
                        let doc_info = DocumentationInfo::from_docsonnet_node(
                            self.cache,
                            documentation_node.body.clone(),
                        );
                        if let Some(doc_info) = doc_info
                            && !doc_info.help_text.is_empty()
                        {
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
