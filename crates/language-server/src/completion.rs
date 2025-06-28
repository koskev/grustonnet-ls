use anyhow::Result;
use lsp_types::{CompletionList, Position};

pub type CompletionResult = Result<CompletionList>;

pub trait Completion: Send {
    fn complete(&self, location: Position, filename: &str) -> CompletionResult;
}
