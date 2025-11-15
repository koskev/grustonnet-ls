use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use grustonnet_config::DuplicateDetectionConfig;
use grustonnet_node::types::{literals::LiteralString, node::Node};
use language_server::diagnostics::DiagnosticsResult;
use lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::diagnostics::{JsonnetDiagnostics, JsonnetDiagnosticsContext};

#[derive(Default)]
pub struct DuplicateValuesDiagnostic {
    pub seen_values: Arc<RwLock<HashMap<u64, Vec<Arc<Node>>>>>,
    pub config: DuplicateDetectionConfig,
}

impl JsonnetDiagnostics for DuplicateValuesDiagnostic {
    fn get_name(&self) -> String {
        "duplicate_values".into()
    }

    fn check_literal_string(
        &self,
        ctx: &JsonnetDiagnosticsContext,
        string: &LiteralString,
    ) -> Option<Vec<DiagnosticsResult>> {
        // Only "real" strings do have a location. All other are part of an index etc
        if ctx.node.node_base.loc_range.is_valid()
            && string.value.len() >= self.config.min_len
            && self.config.min_occurrences > 0
        {
            let mut hasher = std::hash::DefaultHasher::new();
            string.value.hash(&mut hasher);

            self.seen_values
                .write()
                .unwrap()
                .entry(hasher.finish())
                .or_default()
                .push(ctx.node.clone());
        }

        None
    }

    /// Collect all duplicates and emit diagnostics
    fn check_after(&self) -> Option<Vec<DiagnosticsResult>> {
        Some(
            self.seen_values
                .read()
                .unwrap()
                .values()
                // Add one to account for the original node
                .filter(|nodes| nodes.len() + 1 > self.config.min_occurrences)
                .flat_map(|nodes| {
                    nodes.iter().map(|node| DiagnosticsResult {
                        diagnostics: Diagnostic {
                            range: node.node_base.loc_range.clone().into(),
                            message: format!(
                                "The value {} occurs {} times. Consider adding a constant",
                                node.get_name(),
                                nodes.len()
                            ),
                            severity: Some(DiagnosticSeverity::HINT),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                })
                .collect(),
        )
    }
}
