use lsp_types::Diagnostic;

use crate::cache::Document;

pub mod eval;

pub trait Diagnostics {
    fn diagnostics(&self, filename: &str) -> Vec<Diagnostic>;
}
