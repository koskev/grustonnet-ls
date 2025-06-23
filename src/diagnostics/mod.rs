use lsp_types::Diagnostic;

pub mod eval;

pub trait Diagnostics {
    fn diagnostics(&self, filename: &str) -> Vec<Diagnostic>;
}
