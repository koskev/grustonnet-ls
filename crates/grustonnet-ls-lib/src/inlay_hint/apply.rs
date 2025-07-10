use anyhow::Result;
use language_server::cache::Cache;
use lsp_types::InlayHint;

use crate::{cache::JsonnetASTGenerator, inlay_hint::Inlay, node::types::node_kind::NodeKind};

pub struct ApplyInlay<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> ApplyInlay<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

// TODO: Like goto definition: Resolve everything except the last node. Resolve the last node until
// function
impl<'a> Inlay for ApplyInlay<'a> {
    fn inlay(&self, filename: &str) -> Result<Vec<InlayHint>> {
        let doc = self.cache.get_document(filename)?;
        let ast = doc.get_ast()?;
        let doc_stack = doc.get_ast()?.get_complete_stack();
        let hints: Vec<InlayHint> = doc_stack
            .stack
            .iter()
            // For every apply node: Complete the node until we find an apply
            // First find the node in the document and get its stack
            .filter_map(|n| {
                let NodeKind::Apply(apply_node) = n.node_kind.as_ref() else {
                    return None;
                };
                let mut temp_stack =
                    ast.get_stack_by_position(&apply_node.target.node_base.loc_range.end);
                let last_node = temp_stack.get_last_unbuilt_node(self.cache).ok()?;
                // TODO: build the last node?
                let NodeKind::Function(found_function) = *last_node.node_kind else {
                    return None;
                };
                let params = found_function.parameters?;
                let names: Vec<&String> = params.iter().map(|p| &p.name.0).collect();

                Some(
                    apply_node
                        .arguments
                        .positional
                        .iter()
                        .enumerate()
                        .map(|(i, apply_param)| InlayHint {
                            position: apply_param.expr.node_base.loc_range.begin.clone().into(),
                            label: lsp_types::InlayHintLabel::String(format!("{}:", names[i])),
                            kind: None,
                            text_edits: None,
                            tooltip: None,
                            padding_left: None,
                            padding_right: Some(true),
                            data: None,
                        })
                        .collect::<Vec<InlayHint>>(),
                )
            })
            .flatten()
            .collect();
        Ok(hints.into())
    }
}
