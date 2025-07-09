use anyhow::Result;
use lsp_types::InlayHint;

pub mod debug;

pub trait Inlay: Send {
    fn inlay(&self, filename: &str) -> Result<Vec<InlayHint>>;
}
