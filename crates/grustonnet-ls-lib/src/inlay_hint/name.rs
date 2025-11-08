use anyhow::Result;
use jsonnet_location::LocationRange;
use language_server::cache::Cache;
use lsp_types::{InlayHint, Range, Uri};

use crate::{cache::JsonnetASTGenerator, inlay_hint::Inlay, node::types::node_kind::NodeKind};

pub struct NameInlay<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
    threshold: i32,
}

impl<'a> NameInlay<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>, threshold: i32) -> Self {
        Self { cache, threshold }
    }
}

impl<'a> Inlay for NameInlay<'a> {
    fn inlay(&self, uri: &Uri, range: Range) -> Result<Vec<InlayHint>> {
        let doc = self.cache.get_document(uri)?;
        let doc_stack = doc.get_ast()?.get_complete_stack();
        let loc_range = LocationRange {
            file_name: uri.path().as_str().to_string(),
            begin: range.start.into(),
            end: range.end.into(),

            ..Default::default()
        };
        let hints: Vec<InlayHint> = doc_stack
            .stack
            .iter()
            .filter(|n| loc_range.in_range(&n.node_base.loc_range.begin))
            .filter_map(|n| {
                if let NodeKind::DesugaredObject(obj) = n.node_kind.as_ref() {
                    Some(obj)
                } else {
                    None
                }
            })
            .flat_map(|obj| &obj.fields)
            .filter(|field| {
                (field.loc_range.end.line - field.loc_range.begin.line) > self.threshold
            })
            .map(|field| {
                let mut line_end_loc = field.loc_range.end.clone();
                // Push it to the end of the line
                line_end_loc.column += 9999;
                InlayHint {
                    position: line_end_loc.into(),
                    padding_right: None,
                    label: lsp_types::InlayHintLabel::String(field.name.get_name()),
                    kind: None,
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    data: None,
                }
            })
            .collect();
        Ok(hints)
    }
}
