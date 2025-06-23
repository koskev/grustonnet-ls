use crate::node::location::Location;

pub mod global;
pub mod keyword;

pub trait Completion {
    fn complete(&self, location: Location, filename: &str) -> lsp_types::CompletionList;
}

#[cfg(test)]
mod global_test;
