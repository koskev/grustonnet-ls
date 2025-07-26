use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Position, Uri};

use crate::{
    cache::JsonnetASTGenerator,
    node::types::{local_bind::LocalBind, node_kind::NodeKind},
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
    fn complete(&self, pos: Position, uri: &Uri) -> CompletionResult {
        let doc = self.cache.get_document(uri).unwrap();

        let stack = doc.get_ast()?.get_stack_by_position(&pos.into());
        let binds: Vec<LocalBind> = stack
            .stack
            .iter()
            .flat_map(|node| match &(*node.node_kind) {
                NodeKind::Local(local) => local.binds.clone(),
                NodeKind::DesugaredObject(obj) => obj.locals.clone(),
                NodeKind::Function(func) => func
                    .parameters
                    .iter()
                    .map(|param| LocalBind {
                        variable: param.name.clone(),
                        ..Default::default()
                    })
                    .collect(),
                _ => {
                    eprintln!("No bind {}", node.node_kind.variant_name());
                    vec![]
                }
            })
            .collect();

        let items = binds
            .iter()
            .filter_map(|bind| {
                match bind.variable.0.as_str() {
                    // Filter out weird "$" in ast
                    "$" => None,
                    _ => Some(CompletionItem {
                        label: bind.variable.0.clone(),
                        kind: Some(
                            bind.body
                                .as_ref()
                                .map(|body| body.node_kind.get_lsp_kind())
                                .unwrap_or(CompletionItemKind::VARIABLE),
                        ),
                        label_details: Some(CompletionItemLabelDetails {
                            description: bind
                                .body
                                .as_ref()
                                .map(|body| body.node_kind.get_node_kind_name().into()),
                            ..Default::default()
                        }),
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
