use language_server::diagnostics::DiagnosticsResult;
use lsp_types::{Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity};

use crate::{
    diagnostics::{JsonnetDiagnostics, JsonnetDiagnosticsContext},
    node::types::{Local, node_kind::NodeKind},
};

#[derive(Default)]
pub struct ShadowVariableDiagnostics {}

impl JsonnetDiagnostics for ShadowVariableDiagnostics {
    fn get_name(&self) -> String {
        "shadow_var".into()
    }

    fn check_local(
        &self,
        ctx: &JsonnetDiagnosticsContext,
        local: &Local,
    ) -> Option<Vec<DiagnosticsResult>> {
        let mut stack = ctx
            .root
            .get_stack_by_position(&ctx.node.node_base.loc_range.begin);
        stack.stack.pop(); // remove self

        // TODO: Currently this only considers locals and not parameters
        let shadowed_variables: Vec<_> = stack
            .stack
            .iter()
            .filter_map(|n| {
                if let NodeKind::Local(local) = n.node_kind.as_ref() {
                    Some((n, local))
                } else {
                    None
                }
            })
            .filter(|stack_local| stack_local.1.get_name() == local.get_name())
            .collect();

        if !shadowed_variables.is_empty() {
            let mut diags = vec![DiagnosticsResult {
                diagnostics: Diagnostic {
                    range: ctx.node.node_base.loc_range.clone().into(),
                    message: "This variable shadows other variables".into(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    related_information: Some(
                        shadowed_variables
                            .iter()
                            .map(|(shadowed_node, _)| DiagnosticRelatedInformation {
                                message: "This variable is shadowed".into(),
                                location: shadowed_node.node_base.loc_range.clone().into(),
                            })
                            .collect(),
                    ),

                    ..Default::default()
                },
                ..Default::default()
            }];
            diags.extend(
                shadowed_variables
                    .iter()
                    .map(|(shadowed_node, _)| DiagnosticsResult {
                        diagnostics: Diagnostic {
                            range: shadowed_node.node_base.loc_range.clone().into(),
                            message: "This variable is shadowed".into(),
                            severity: Some(DiagnosticSeverity::INFORMATION),
                            related_information: Some(vec![DiagnosticRelatedInformation {
                                location: ctx.node.node_base.loc_range.clone().into(),
                                message: "This variable shadows other variables".into(),
                            }]),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
            Some(diags)
        } else {
            None
        }
    }
}
