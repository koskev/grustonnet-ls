use anyhow::Result;
use jsonnet_location::LocationRange;
use language_server::cache::Cache;
use lsp_types::{InlayHint, Range, Uri};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{cache::JsonnetASTGenerator, inlay_hint::Inlay, node::NodeHelper};

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
                let (apply_node, found_function) = n.get_apply_function(ast.clone(), self.cache)?;
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
                                    "{}=",
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
