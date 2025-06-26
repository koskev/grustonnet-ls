use lsp_types::Diagnostic;

pub trait Diagnostics {
    fn diagnostics(&self, filename: &str) -> Vec<Diagnostic>;
}
