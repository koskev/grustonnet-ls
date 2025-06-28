use jsonnet_std_docs::StdFunctions;
use language_server::completion::{Completion, CompletionResult};
use lsp_types::{CompletionItem, CompletionList, Documentation, Position};

const STDLIB_DEFINITIONS: &'static str = include_str!(concat!(env!("OUT_DIR"), "/stdlib.json"));

pub struct StdCompletion;

impl StdCompletion {
    pub fn new() -> Self {
        Self {}
    }
}

// TODO: Remove this dedicated completion and instead support documentation for functions and
// handle the stdlib as a normal function with documentation
impl Completion for StdCompletion {
    fn complete(&self, _location: Position, _filename: &str) -> CompletionResult {
        let functions = StdFunctions::generate(STDLIB_DEFINITIONS);
        let items = functions
            .functions
            .iter()
            .map(|(_name, func)| {
                let param_string = match &func.params {
                    Some(list) => format!("({})", list.join(", ")),
                    None => "".to_string(),
                };
                CompletionItem {
                    label: func.name.clone(),
                    detail: Some(format!("{}{}", func.name.clone(), param_string)),
                    documentation: Some(Documentation::String(func.description.clone())),
                    ..Default::default()
                }
            })
            .collect();

        Ok(CompletionList {
            is_incomplete: false,
            items,
        })
    }
}
