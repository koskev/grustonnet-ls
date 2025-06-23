use std::str::FromStr;

use lsp_types::{CodeDescription, Diagnostic, DiagnosticSeverity, Range, Uri};

use crate::{
    bridge::ast::{GenerateAST, GoJsonnet},
    cache::Cache,
    diagnostics::Diagnostics,
};

pub struct EvalDiagnostics<'a> {
    cache: &'a Cache,
}

impl<'a> EvalDiagnostics<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

impl<'a> Diagnostics for EvalDiagnostics<'a> {
    fn diagnostics(&self, filename: &str) -> Vec<lsp_types::Diagnostic> {
        let doc = self.cache.get_document(filename).unwrap();
        let res = GoJsonnet::new().evaluate_snippet(&doc.filename, &doc.content);
        eprintln!("Diagnostics: {:?}", res);

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
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            }];
        }

        vec![]
    }
}
