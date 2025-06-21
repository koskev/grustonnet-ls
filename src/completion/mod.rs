use crate::node::location::Location;

pub mod global;

pub trait Completion {
    fn complete(&self, location: Location, filename: &str) -> lsp_types::CompletionList;
}
