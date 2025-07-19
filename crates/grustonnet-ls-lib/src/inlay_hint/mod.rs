use anyhow::Result;
use lsp_types::{InlayHint, Range, Uri};

pub mod apply;
pub mod debug;

pub trait Inlay: Send {
    fn inlay(&self, uri: &Uri, range: Range) -> Result<Vec<InlayHint>>;
}
