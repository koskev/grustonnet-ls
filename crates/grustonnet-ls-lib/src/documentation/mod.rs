use std::sync::Arc;

use language_server::{cache::Cache, utils::UriHelper};
use lsp_types::Uri;

use crate::{
    cache::JsonnetASTGenerator,
    completion::local::CallStackIter,
    node::types::{index::Index, literals::LiteralString, node::Node, node_kind::NodeKind},
};

pub struct DocumentationInfo {
    pub help_text: String,
}

impl DocumentationInfo {
    fn compile_object(cache: &Cache<JsonnetASTGenerator>, node: Arc<Node>) -> Option<Arc<Node>> {
        let doc = cache
            .get_document(&Uri::from_path(&node.node_base.loc_range.file_name).ok()?)
            .ok()?;
        let mut doc_stack = doc
            .get_ast()
            .ok()?
            .get_stack_by_position(&node.node_base.loc_range.begin);
        let iter = CallStackIter::new(cache, &mut doc_stack)?;
        let last_node = iter.last();

        None
    }

    fn resolve_indices(
        cache: &Cache<JsonnetASTGenerator>,
        node: Arc<Node>,
        indices: &[&str],
    ) -> Option<Arc<Node>> {
        let documentation_doc = cache
            .get_document(&Uri::from_path(&node.node_base.loc_range.file_name).ok()?)
            .ok()?;
        // Got the correct documentation string
        // Now just resolve it
        let mut doc_stack = documentation_doc
            .get_ast()
            .ok()?
            .get_stack_by_position(&node.node_base.loc_range.begin);
        let mut prev_node = node.clone();
        for index in indices {
            prev_node = Node {
                node_kind: Box::new(NodeKind::Index(Index {
                    target: prev_node.clone(),
                    index: Arc::new(Node {
                        node_kind: Box::new(NodeKind::LiteralString(LiteralString {
                            value: index.to_string(),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into();

            doc_stack.push(prev_node.clone());
        }
        let iter = CallStackIter::new(cache, &mut doc_stack)?;
        iter.last()
    }

    pub fn from_docsonnet_node(
        cache: &Cache<JsonnetASTGenerator>,
        documentation_node: Arc<Node>,
    ) -> Option<Self> {
        let node = Self::resolve_indices(cache, documentation_node, &["function", "help"])?;

        Some(Self {
            help_text: node.get_name(),
        })
    }
}
