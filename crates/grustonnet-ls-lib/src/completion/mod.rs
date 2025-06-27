use anyhow::Result;
use lsp_types::CompletionList;

use crate::node::location::Location;

pub mod global;
pub mod keyword;
pub mod local;
pub mod std;

type CompletionResult = Result<CompletionList>;

pub trait Completion {
    fn complete(&self, location: Location, filename: &str) -> CompletionResult;
}
