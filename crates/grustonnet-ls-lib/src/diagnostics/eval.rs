use language_server::{
    cache::Cache,
    diagnostics::{Diagnostics, DiagnosticsResult},
};
use lsp_types::{CodeDescription, Diagnostic, DiagnosticSeverity, Range, Uri};

use crate::{bridge::GenerateAST, cache::JsonnetASTGenerator};

pub struct EvalDiagnostics {
    cache: Cache<JsonnetASTGenerator>,
}

impl EvalDiagnostics {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl Diagnostics for EvalDiagnostics {
    fn get_name(&self) -> String {
        "EvalDiagnostics".into()
    }
    fn diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult> {
        let doc = self.cache.get_document(uri).unwrap();
        let res = self
            .cache
            .ast_generator
            .jsonnet
            .evaluate_snippet(&doc.filename, &doc.content);

        if let Err(diag_err) = res {
            return vec![
                Diagnostic {
                    range: Range {
                        start: diag_err.start.into(),
                        end: diag_err.end.into(),
                    },
                    message: diag_err.message,
                    code_description: Some(CodeDescription { href: uri.clone() }),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }
                .into(),
            ];
        }

        vec![]
    }
}
