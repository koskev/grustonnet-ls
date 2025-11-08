use language_server::{
    cache::Cache,
    diagnostics::{Diagnostics, DiagnosticsResult},
};
use lsp_types::Uri;

use crate::{
    cache::JsonnetASTGenerator,
    node::types::{Local, node_kind::NodeKind},
};

pub mod cst_linters;
pub mod eval;
pub mod go_lint;
pub mod linters;

pub struct JsonnetDiagnosticsContext {
    cache: Cache<JsonnetASTGenerator>,
    uri: Uri,
}

macro_rules! add_diag {
    ($name: ident, $($v: ident: $t: ty ),*) => {
        #[allow(unused)]
        fn $name(&self, ctx: &JsonnetDiagnosticsContext, $($v: $t),*) -> Option<Vec<DiagnosticsResult>> { None }
    }
}

/// This trait provides functions for jsonnet specific linters
pub trait JsonnetDiagnostics: Send + Sync {
    add_diag!(check_local, local: &Local);

    fn get_name(&self) -> String;
}

#[derive(Default)]
pub struct DiagnosticsHandler {
    pub cache: Cache<JsonnetASTGenerator>,
    pub diags: Vec<Box<dyn JsonnetDiagnostics>>,
}

impl Diagnostics for DiagnosticsHandler {
    fn diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult> {
        let mut result = vec![];
        let Ok(doc) = self.cache.get_document(uri) else {
            return result;
        };
        let Ok(ast) = doc.get_ast() else {
            return result;
        };

        let ctx = JsonnetDiagnosticsContext {
            cache: self.cache.clone(),
            uri: uri.clone(),
        };
        for node in ast.get_complete_stack().stack.iter() {
            for diag in self.diags.iter() {
                let res = match node.node_kind.as_ref() {
                    NodeKind::Local(local) => diag.check_local(&ctx, local),
                    _ => None,
                };
                if let Some(res) = res {
                    result.extend(res);
                }
            }
        }
        result
    }

    fn get_name(&self) -> String {
        "jsonnet_lints".into()
    }
}
