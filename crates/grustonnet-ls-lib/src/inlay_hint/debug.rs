use anyhow::Result;
use language_server::cache::Cache;
use lsp_types::InlayHint;

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
    fn inlay(&self, filename: &str) -> Result<Vec<InlayHint>> {
        let doc = self.cache.get_document(filename)?;
        let doc_stack = doc.get_ast()?.get_complete_stack();
        let hints: Vec<InlayHint> = doc_stack
            .stack
            .iter()
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
        Ok(hints.into())
    }
}
