use lsp_types::Diagnostic;

pub mod eval;
pub mod lint;

pub trait Diagnostics {
    fn diagnostics(&self, filename: &str) -> Vec<Diagnostic>;
}
