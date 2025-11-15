use std::collections::HashMap;

use grustonnet_node::types::{Local, node_kind::NodeKind};
use jsonnet_location::Location;
use language_server::{
    cache::Cache,
    diagnostics::{Diagnostics, DiagnosticsResult},
};
use lsp_types::{
    CodeActionKind, CodeDescription, Diagnostic, DiagnosticSeverity, DiagnosticTag, Range,
    TextEdit, Uri, WorkspaceEdit,
};

use crate::{cache::JsonnetASTGenerator, references::ReferenceProvider};

pub struct UnusedDiagnostics {
    cache: Cache<JsonnetASTGenerator>,
}

impl UnusedDiagnostics {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl UnusedDiagnostics {
    fn get_code_action(&self, uri: &Uri, local: &Local) -> Option<Vec<lsp_types::CodeAction>> {
        let mut pos = local.get_identifier_position()?;
        let name = local.get_name()?;
        pos.end = Location {
            line: pos.begin.line,
            column: pos.begin.column + name.len() as i32,
        };
        Some(vec![lsp_types::CodeAction {
            title: format!("Mark variable \"{}\" as unused", name),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(
                    uri.clone(),
                    vec![TextEdit {
                        new_text: format!("_{}", name),
                        range: Range {
                            start: pos.begin.into(),
                            end: pos.end.into(),
                        },
                    }],
                )])),
                ..Default::default()
            }),
            ..Default::default()
        }])
    }
    fn get_diagnostics(&self, uri: &Uri) -> Option<Vec<DiagnosticsResult>> {
        let doc = self.cache.get_document(uri).unwrap();
        let stack = doc.get_ast().ok()?.get_complete_stack();
        let locals = stack
            .stack
            .iter()
            .filter_map(|n| {
                if let NodeKind::Local(loc) = n.node_kind.as_ref() {
                    Some(loc)
                } else {
                    None
                }
            })
            .filter(|loc| !loc.get_name().unwrap_or_default().starts_with("_"));

        let search_paths = vec![];
        let provider = ReferenceProvider::new(&self.cache, &search_paths);

        Some(
            locals
                .filter(|local| {
                    // TODO: There has to be some Rust magic for this
                    if let Some(range) = local.get_identifier_position()
                        && let Ok(res) = provider.references(range.begin.clone(), uri, true)
                        && let Some(locations) = res
                        && locations.len() == 1
                    {
                        true
                    } else {
                        false
                    }
                })
                .filter_map(|local| {
                    Some(DiagnosticsResult {
                        diagnostics: Diagnostic {
                            range: Range {
                                start: local.get_identifier_position()?.begin.clone().into(),
                                end: local.get_identifier_position()?.end.clone().into(),
                            },
                            message: format!(
                                "Unused variable. If this is intentional prefix with an underscore: _{}",
                                local.get_name().unwrap_or("<variable>".to_string())
                            ),
                            code_description: Some(CodeDescription { href: uri.clone() }),
                            severity: Some(DiagnosticSeverity::WARNING),
                            tags: Some(vec![
                                DiagnosticTag::UNNECESSARY,
                            ]),
                            ..Default::default()
                        },
                        code_actions: self.get_code_action(uri, local).unwrap_or_default(),
                        ..Default::default()
                    })
                })
                .collect(),
        )
    }
}

impl Diagnostics for UnusedDiagnostics {
    fn get_name(&self) -> String {
        "lint".into()
    }
    fn diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult> {
        self.get_diagnostics(uri).unwrap_or_default()
    }
}
