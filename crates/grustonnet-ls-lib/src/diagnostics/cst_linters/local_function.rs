use language_server::{
    cache::Cache,
    diagnostics::{Diagnostics, DiagnosticsResult},
    utils::cst::CstNodeHelper,
};
use lsp_types::{Diagnostic, DiagnosticSeverity, Range};
use tree_sitter::{Query, QueryCursor};

use crate::{cache::JsonnetASTGenerator, cst::new_tree, node::location::Location};

pub struct LocalFunctionDiagnostics {
    pub cache: Cache<JsonnetASTGenerator>,
}

impl Diagnostics for LocalFunctionDiagnostics {
    fn get_name(&self) -> String {
        "local_function".into()
    }

    fn diagnostics(
        &self,
        uri: &lsp_types::Uri,
    ) -> Vec<language_server::diagnostics::DiagnosticsResult> {
        let mut results = vec![];
        let Ok(doc) = self.cache.get_document(uri) else {
            return vec![];
        };
        let Some(tree) = new_tree(&doc.content) else {
            return vec![];
        };
        let query_source = "(bind (id) @id (anonymous_function) @func) @bind";
        let query = Query::new(&tree.language(), query_source).expect("BUG: Invalid query");
        let mut cursor = QueryCursor::new();
        let captures = cursor.matches(&query, tree.root_node(), doc.content.as_bytes());

        for cap in captures {
            // Due to the query these unwraps won't crash
            let id = cap
                .captures
                .iter()
                .find(|c| c.index == query.capture_index_for_name("id").unwrap())
                .unwrap();
            let bind = cap
                .captures
                .iter()
                .find(|c| c.index == query.capture_index_for_name("bind").unwrap())
                .unwrap();
            let start: Location = bind.node.start_position().into();
            let end: Location = bind.node.end_position().into();
            let name = id.node.get_name(&doc.content).unwrap_or_default();
            results.push(DiagnosticsResult {
                diagnostics: Diagnostic {
                    range: Range {
                        start: start.into(),
                        end: end.into(),
                    },
                    message: format!(
                        "Instead of local {} = function() <body>, write local {}() = <body>",
                        name, name
                    ),
                    severity: Some(DiagnosticSeverity::HINT),
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        results
    }
}
