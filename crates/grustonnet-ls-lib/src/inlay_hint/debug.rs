use anyhow::Result;
use jsonnet_location::LocationRange;
use language_server::cache::Cache;
use lsp_types::{InlayHint, Range, Uri};

use crate::{cache::JsonnetASTGenerator, inlay_hint::Inlay};

pub struct DebugInlay<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> DebugInlay<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> Inlay for DebugInlay<'a> {
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
            .map(|n| InlayHint {
                position: n.node_base.loc_range.begin.clone().into(),
                padding_right: Some(true),
                label: lsp_types::InlayHintLabel::String(n.node_kind.variant_name().to_string()),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: None,
                data: None,
            })
            .collect();
        Ok(hints)
    }
}
