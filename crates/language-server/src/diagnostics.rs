use lsp_types::{Diagnostic, Uri};

pub trait Diagnostics {
    fn diagnostics(&self, uri: &Uri) -> Vec<Diagnostic>;
}
