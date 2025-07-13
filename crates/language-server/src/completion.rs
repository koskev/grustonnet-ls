use anyhow::Result;
use lsp_types::{CompletionList, Position, Uri};

pub type CompletionResult = Result<CompletionList>;

pub trait Completion: Send {
    fn complete(&self, location: Position, uri: &Uri) -> CompletionResult;
}
