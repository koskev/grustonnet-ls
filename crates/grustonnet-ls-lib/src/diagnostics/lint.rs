use language_server::{cache::Cache, diagnostics::Diagnostics};
use lsp_types::{CodeDescription, Diagnostic, DiagnosticSeverity, Range, Uri};

use crate::{bridge::GenerateAST, cache::JsonnetASTGenerator};

pub struct LintDiagnostics<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> LintDiagnostics<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> Diagnostics for LintDiagnostics<'a> {
    fn diagnostics(&self, uri: &Uri) -> Vec<lsp_types::Diagnostic> {
        let doc = self.cache.get_document(uri).unwrap();
        let res = self
            .cache
            .ast_generator
            .jsonnet
            .lint_snippet(&doc.filename, &doc.content);

        if let Err(diag_err) = res {
            return vec![Diagnostic {
                range: Range {
                    start: diag_err.start.into(),
                    end: diag_err.end.into(),
                },
                message: diag_err.message,
                code_description: Some(CodeDescription { href: uri.clone() }),
                severity: Some(DiagnosticSeverity::WARNING),
                ..Default::default()
            }];
        }

        vec![]
    }
}
