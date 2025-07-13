use anyhow::Result;
use lsp_types::{InlayHint, Uri};

pub mod apply;
pub mod debug;

pub trait Inlay: Send {
    fn inlay(&self, uri: &Uri) -> Result<Vec<InlayHint>>;
}
