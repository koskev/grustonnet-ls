use lsp_types::{CompletionItem, CompletionList};

use crate::{completion::Completion, node::location::Location, stdlib::StdFunctions};

pub struct StdCompletion;

impl StdCompletion {
    pub fn new() -> Self {
        Self {}
    }
}

// TODO: Remove this dedicated completion and instead support documentation for functions and
// handle the stdlib as a normal function with documentation
impl Completion for StdCompletion {
    fn complete(&self, _location: Location, _filename: &str) -> CompletionList {
        let functions = StdFunctions::generate();
        let items = functions
            .functions
            .iter()
            .map(|(_name, func)| CompletionItem {
                label: func.name.clone(),
                detail: Some(func.description.clone()),
                ..Default::default()
            })
            .collect();

        CompletionList {
            is_incomplete: false,
            items,
        }
    }
}
