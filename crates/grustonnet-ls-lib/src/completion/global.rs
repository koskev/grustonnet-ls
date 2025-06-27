use language_server::cache::Cache;
use lsp_types::{CompletionItem, CompletionItemKind};

use crate::{
    cache::JsonnetASTGenerator,
    completion::{Completion, CompletionResult},
    node::{LocalBind, NodeKind, location::Location},
};

pub struct GlobalCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> GlobalCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> Completion for GlobalCompletion<'a> {
    fn complete(&self, pos: Location, filename: &str) -> CompletionResult {
        let doc = self.cache.get_document(filename).unwrap();

        let stack = doc.get_ast()?.get_stack_by_position(&pos);
        let binds: Vec<LocalBind> = stack
            .stack
            .iter()
            .flat_map(|node| match &(*node.node_kind) {
                NodeKind::Local(local) => local.binds.clone(),
                NodeKind::DesugaredObject(obj) => obj.locals.clone(),
                NodeKind::Function(func) => match &func.parameters {
                    Some(params) => params
                        .iter()
                        .map(|param| LocalBind {
                            variable: param.name.clone(),
                            ..Default::default()
                        })
                        .collect(),
                    None => vec![],
                },
                _ => {
                    eprintln!("No bind {}", node.node_kind.variant_name());
                    vec![]
                }
            })
            .collect();

        let items = binds
            .iter()
            .filter_map(|bind| {
                let kind = match &bind.body {
                    Some(kind) => {
                        if let NodeKind::Function(_) = *kind.node_kind {
                            CompletionItemKind::FUNCTION
                        } else {
                            CompletionItemKind::VARIABLE
                        }
                    }
                    None => CompletionItemKind::VARIABLE,
                };
                match bind.variable.0.as_str() {
                    // Filter out weird "$" in ast
                    "$" => None,
                    _ => Some(CompletionItem {
                        label: bind.variable.0.clone(),
                        kind: Some(kind),
                        detail: match &bind.body {
                            Some(body) => Some(body.node_kind.variant_name().to_string()),
                            None => None,
                        },
                        ..Default::default()
                    }),
                }
            })
            .collect();
        Ok(lsp_types::CompletionList {
            items,
            is_incomplete: false,
        })
    }
}
