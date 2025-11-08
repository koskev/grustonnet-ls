use anyhow::Result;
use jsonnet_location::LocationRange;
use language_server::cache::Cache;
use lsp_types::{InlayHint, Range, Uri};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{cache::JsonnetASTGenerator, inlay_hint::Inlay, node::types::node_kind::NodeKind};

pub struct ApplyInlay<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> ApplyInlay<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> Inlay for ApplyInlay<'a> {
    fn inlay(&self, uri: &Uri, range: Range) -> Result<Vec<InlayHint>> {
        let doc = self.cache.get_document(uri)?;
        let ast = doc.get_ast()?;
        let doc_stack = doc.get_ast()?.get_complete_stack();
        let loc_range = LocationRange {
            file_name: uri.path().as_str().to_string(),
            begin: range.start.into(),
            end: range.end.into(),
            ..Default::default()
        };
        let hints: Vec<InlayHint> = doc_stack
            .stack
            .into_par_iter()
            .filter(|n| loc_range.in_range(&n.node_base.loc_range.begin))
            // For every apply node: Complete the node until we find an apply
            // First find the node in the document and get its stack
            .filter_map(|n| {
                let NodeKind::Apply(apply_node) = n.node_kind.as_ref() else {
                    return None;
                };
                let mut temp_stack =
                    ast.get_stack_by_position(&apply_node.target.node_base.loc_range.end);
                // TODO: If we have a().b().c().d() we will build the node way more than needed
                let last_node = temp_stack.get_last_unbuilt_node(self.cache).ok()?;
                // TODO: build the last node?
                let NodeKind::Function(found_function) = last_node.node_kind.as_ref() else {
                    return None;
                };
                let params = &found_function.parameters;
                let names: Vec<&String> = params.iter().map(|p| &p.name.0).collect();

                Some(
                    apply_node
                        .arguments
                        .positional
                        .iter()
                        .enumerate()
                        .filter_map(|(i, apply_param)| {
                            Some(InlayHint {
                                position: apply_param.expr.node_base.loc_range.begin.clone().into(),
                                label: lsp_types::InlayHintLabel::String(format!(
                                    "{}:",
                                    // this probably happens for $std or a top level function without
                                    // any params. e.g. in crates/grustonnet-ls-lib/testdata/complete/import/nested_func.libsonnet
                                    names.get(i)?
                                )),
                                kind: None,
                                text_edits: None,
                                tooltip: None,
                                padding_left: None,
                                padding_right: Some(true),
                                data: None,
                            })
                        })
                        .collect::<Vec<InlayHint>>(),
                )
            })
            .flatten()
            .collect();
        Ok(hints)
    }
}
