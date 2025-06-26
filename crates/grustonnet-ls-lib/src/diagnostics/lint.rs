use std::str::FromStr;

use language_server::diagnostics::Diagnostics;
use lsp_types::{CodeDescription, Diagnostic, DiagnosticSeverity, Range, Uri};

use crate::{
    bridge::{GenerateAST, GoJsonnet},
    cache::Cache,
};

pub struct LintDiagnostics<'a> {
    cache: &'a Cache,
}

impl<'a> LintDiagnostics<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

impl<'a> Diagnostics for LintDiagnostics<'a> {
    fn diagnostics(&self, filename: &str) -> Vec<lsp_types::Diagnostic> {
        let doc = self.cache.get_document(filename).unwrap();
        let res = GoJsonnet::new().lint_snippet(&doc.filename, &doc.content);

        if let Err(diag_err) = res {
            return vec![Diagnostic {
                range: Range {
                    start: diag_err.start.into(),
                    end: diag_err.end.into(),
                },
                message: diag_err.message,
                code_description: Some(CodeDescription {
                    href: Uri::from_str(filename).unwrap(),
                }),
                severity: Some(DiagnosticSeverity::WARNING),
                ..Default::default()
            }];
        }

        vec![]
    }
}
