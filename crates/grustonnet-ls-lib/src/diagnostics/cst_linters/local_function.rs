use std::collections::HashMap;

use jsonnet_cst::new_tree;
use jsonnet_location::Location;
use language_server::{
    cache::Cache,
    diagnostics::{Diagnostics, DiagnosticsResult},
    utils::cst::CstNodeHelper,
};
use lsp_types::{
    CodeAction, CodeActionKind, Diagnostic, DiagnosticSeverity, Range, TextEdit, Uri, WorkspaceEdit,
};
use tree_sitter::{Query, QueryCursor, QueryMatch};

use crate::cache::JsonnetASTGenerator;

pub struct LocalFunctionDiagnostics {
    pub cache: Cache<JsonnetASTGenerator>,
}

impl LocalFunctionDiagnostics {
    fn handle_query(
        &self,
        uri: &Uri,
        cap: &QueryMatch,
        query: &Query,
        content: &str,
    ) -> Option<Vec<language_server::diagnostics::DiagnosticsResult>> {
        let mut results = vec![];
        // Due to the query these unwraps won't crash
        let id = cap
            .captures
            .iter()
            .find(|c| c.index == query.capture_index_for_name("id").unwrap())?;
        let bind = cap
            .captures
            .iter()
            .find(|c| c.index == query.capture_index_for_name("bind").unwrap())?;
        let params = cap
            .captures
            .iter()
            .find(|c| c.index == query.capture_index_for_name("params").unwrap());
        let params_end = cap
            .captures
            .iter()
            .find(|c| c.index == query.capture_index_for_name("params_end").unwrap())?;
        let start: Location = bind.node.start_position().into();
        let end: Location = bind.node.end_position().into();
        let name = id.node.get_name(content)?;
        let params_content = match params {
            None => String::new(),
            Some(params) => params.node.get_name(content)?,
        };
        let bind_start: Location = bind.node.start_position().into();
        let params_end: Location = params_end.node.end_position().into();
        results.push(DiagnosticsResult {
            diagnostics: Diagnostic {
                range: Range {
                    start: start.into(),
                    end: end.into(),
                },
                message: format!(
                    "Instead of local {name} = function({params_content}) <body>, write local {name}({params_content}) = <body>",
                ),
                severity: Some(DiagnosticSeverity::HINT),
                ..Default::default()
            },
            code_actions: vec![CodeAction {
                title: "Refactor local function".into(),
                kind: Some(CodeActionKind::REFACTOR),
                edit: Some(WorkspaceEdit {
                    changes: Some(HashMap::from([(
                        uri.clone(),
                        vec![TextEdit {
                            new_text: format!("{}({}) = ", name, params_content),
                            range: Range {
                                start: bind_start.into(),
                                end: params_end.into(),
                            },
                        }],
                    )])),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        });

        Some(results)
    }
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
            return results;
        };
        let Some(tree) = new_tree(&doc.content) else {
            return results;
        };
        let query_source = r#"(bind (id) @id (anonymous_function (params)? @params (")") @params_end) @func) @bind"#;
        let query = Query::new(&tree.language(), query_source).expect("BUG: Invalid query");
        let mut cursor = QueryCursor::new();
        let captures = cursor.matches(&query, tree.root_node(), doc.content.as_bytes());

        for cap in captures {
            if let Some(result) = self.handle_query(uri, &cap, &query, &doc.content) {
                results.extend(result);
            }
        }
        results
    }
}
