use language_server::{cache::Cache, diagnostics::Diagnostics};
use lsp_types::{CodeDescription, Diagnostic, DiagnosticSeverity, Range, Uri};

use crate::{bridge::GenerateAST, cache::JsonnetASTGenerator};

pub struct LintDiagnostics {
    cache: Cache<JsonnetASTGenerator>,
}

impl LintDiagnostics {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl Diagnostics for LintDiagnostics {
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
