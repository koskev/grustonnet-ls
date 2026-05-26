use std::collections::HashMap;

use anyhow::{Result, anyhow};
use grustonnet_node::types::node_kind::NodeKind;
use jsonnet_location::Range;
use language_server::cache::Cache;
use lsp_types::{CodeActionKind, TextEdit, Uri, WorkspaceEdit};

use crate::{cache::JsonnetASTGenerator, node::NodeHelper};

pub trait CodeAction {
    fn get_code_actions(&self, uri: &Uri, range: Range) -> Result<Vec<lsp_types::CodeAction>>;
}

pub struct ParameterCodeAction {
    cache: Cache<JsonnetASTGenerator>,
}

impl ParameterCodeAction {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl CodeAction for ParameterCodeAction {
    fn get_code_actions(&self, uri: &Uri, range: Range) -> Result<Vec<lsp_types::CodeAction>> {
        let doc = self.cache.get_document(uri)?;
        let ast = doc.get_ast()?;
        let stack = ast.get_stack_by_position(&range.begin);
        let apply_node = stack
            .iter()
            .find(|node| matches!(node.node_kind.as_ref(), NodeKind::Apply(_)))
            .ok_or(anyhow!("No apply found"))?;

        let apply_function_data = apply_node
            .get_apply_function(ast.clone(), &self.cache)
            .ok_or(anyhow!("No apply function found"))?;
        let params = &apply_function_data.function.parameters;
        let names: Vec<&String> = params.iter().map(|p| &p.name.0).collect();

        Ok(vec![lsp_types::CodeAction {
            title: "Add explicit parameter names".into(),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(
                    uri.clone(),
                    apply_function_data
                        .apply
                        .arguments
                        .positional
                        .iter()
                        .enumerate()
                        .filter_map(|(i, param)| {
                            let name = names.get(i)?.to_string();
                            Some((name, param))
                        })
                        .map(|(name, param)| {
                            let pos: lsp_types::Position =
                                param.clone().expr.node_base.loc_range.begin.clone().into();
                            TextEdit {
                                new_text: format!("{name}="),
                                range: lsp_types::Range {
                                    start: pos,
                                    end: pos,
                                },
                                ..Default::default()
                            }
                        })
                        .collect(),
                )])),
                ..Default::default()
            }),
            ..Default::default()
        }])
    }
}
