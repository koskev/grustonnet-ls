use jsonnet_cst::new_tree;
use jsonnet_location::{Location, LocationRange};
use language_server::{
    cache::Cache,
    diagnostics::{DiagnosticFilter, DiagnosticsResult},
};
use lsp_types::Uri;
use tree_sitter::{Query, QueryCursor};

use crate::cache::JsonnetASTGenerator;

#[derive(Clone)]
pub struct JsonnetDiagnosticFilter {
    pub cache: Cache<JsonnetASTGenerator>,
}

impl JsonnetDiagnosticFilter {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }

    fn should_ignore(&self, uri: &Uri, loc_range: LocationRange) -> Option<bool> {
        let doc = self.cache.get_document(uri).ok()?;
        let tree = new_tree(&doc.content)?;
        let query_source = r#"
            (
                (comment) @comment (#eq? @comment "// nolint")
                (local_bind (local) (bind) @local_sibling)?
                ((_) @generic_sibling)?
            )
        "#;
        let query = Query::new(&tree.language(), query_source).expect("BUG: Invalid query");

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.matches(&query, tree.root_node(), doc.content.as_bytes());

        Some(captures.any(|cap| {
            let Some(sibling) = cap.captures.iter().find(|c| {
                c.index
                    == query
                        .capture_index_for_name("local_sibling")
                        .unwrap_or(query.capture_index_for_name("generic_sibling").unwrap())
            }) else {
                return false;
            };
            let sibling_begin: Location = sibling.node.start_position().into();
            let sibling_end: Location = sibling.node.end_position().into();
            let sibling_range = LocationRange {
                begin: sibling_begin,
                end: sibling_end,
                ..Default::default()
            };
            sibling_range.in_range(&loc_range.begin)
        }))
    }
}

impl DiagnosticFilter for JsonnetDiagnosticFilter {
    fn filter_diagnostics(
        &self,
        uri: &Uri,
        results: Vec<DiagnosticsResult>,
    ) -> Vec<DiagnosticsResult> {
        results
            .into_iter()
            .filter(|result| {
                !self
                    .should_ignore(uri, result.diagnostics.range.into())
                    .unwrap_or(false)
            })
            .collect()
    }
}
