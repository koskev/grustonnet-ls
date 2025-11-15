use std::{collections::HashMap, marker::PhantomData};

use grustonnet_node::types::Local;
use jsonnet_location::Location;
use language_server::diagnostics::DiagnosticsResult;
use lsp_types::{
    CodeActionKind, Diagnostic, DiagnosticSeverity, Range, TextEdit, Uri, WorkspaceEdit,
};

use crate::diagnostics::{JsonnetDiagnostics, JsonnetDiagnosticsContext};

pub trait VariableNaming: Send + Sync {
    fn rename(input: &str) -> String;
    fn name() -> String;
}

pub struct SnakeCaseDiagnostics {}

impl VariableNaming for SnakeCaseDiagnostics {
    fn name() -> String {
        "snake_case".into()
    }
    fn rename(input: &str) -> String {
        input
            .chars()
            .flat_map(|c| {
                if c.is_uppercase() {
                    vec!['_', c.to_ascii_lowercase()]
                } else {
                    vec![c]
                }
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct VariableNamingDiagnostics<T>
where
    T: VariableNaming,
{
    phantom: PhantomData<T>,
}

impl<T: VariableNaming> VariableNamingDiagnostics<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    // TODO: use lsp rename
    fn get_code_action(
        &self,
        uri: &Uri,
        local: &Local,
        new_name: &str,
    ) -> Option<Vec<lsp_types::CodeAction>> {
        let mut pos = local.get_identifier_position()?;
        let name = local.get_name()?;
        pos.end = Location {
            line: pos.begin.line,
            column: pos.begin.column + name.len() as i32,
        };
        Some(vec![lsp_types::CodeAction {
            title: format!("Rename variable \"{}\" to \"{}\"", name, new_name),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(
                    uri.clone(),
                    vec![TextEdit {
                        new_text: new_name.into(),
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
}

impl<T: VariableNaming> JsonnetDiagnostics for VariableNamingDiagnostics<T> {
    fn get_name(&self) -> String {
        "variable_naming".into()
    }

    fn check_local(
        &self,
        ctx: &JsonnetDiagnosticsContext,
        local: &Local,
    ) -> Option<Vec<DiagnosticsResult>> {
        let name = local.get_name()?;
        let new_name = T::rename(&name);
        if name == new_name {
            return None;
        }
        Some(vec![DiagnosticsResult {
            diagnostics: Diagnostic {
                range: Range {
                    start: local.get_identifier_position()?.begin.clone().into(),
                    end: local.get_identifier_position()?.end.clone().into(),
                },
                message: format!(
                    "Variable is not in {}. Change it to {}",
                    T::name(),
                    new_name
                ),
                severity: Some(DiagnosticSeverity::WARNING),
                ..Default::default()
            },
            code_actions: self
                .get_code_action(&ctx.uri, local, &new_name)
                .unwrap_or_default(),
            ..Default::default()
        }])
    }
}
