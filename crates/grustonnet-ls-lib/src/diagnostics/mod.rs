use std::sync::Arc;

use language_server::{
    cache::Cache,
    diagnostics::{Diagnostics, DiagnosticsResult},
};
use lsp_types::Uri;

use crate::{
    cache::JsonnetASTGenerator,
    node::types::{
        Local,
        function::{Apply, Function},
        node::Node,
        node_kind::NodeKind,
        var::Var,
    },
};

pub mod cst_linters;
pub mod eval;
pub mod filter;
pub mod go_lint;
pub mod linters;

pub struct JsonnetDiagnosticsContext {
    cache: Cache<JsonnetASTGenerator>,
    uri: Uri,

    /// The generic node the function uses
    node: Arc<Node>,
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
    add_diag!(check_var, var: &Var);
    add_diag!(check_apply, apply: &Apply);
    add_diag!(check_function, function: &Function);

    fn get_name(&self) -> String;
}

#[derive(Default)]
pub struct ASTDiagnosticsHandler {
    pub cache: Cache<JsonnetASTGenerator>,
    pub diags: Vec<Box<dyn JsonnetDiagnostics>>,
}

impl Diagnostics for ASTDiagnosticsHandler {
    fn diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult> {
        let mut result = vec![];
        let Ok(doc) = self.cache.get_document(uri) else {
            return result;
        };
        let Ok(ast) = doc.get_ast() else {
            return result;
        };

        for node in ast.get_complete_stack().stack.iter() {
            let ctx = JsonnetDiagnosticsContext {
                cache: self.cache.clone(),
                uri: uri.clone(),
                node: node.clone(),
            };
            for diag in self.diags.iter() {
                let res = match node.node_kind.as_ref() {
                    NodeKind::Local(local) => diag.check_local(&ctx, local),
                    NodeKind::Var(var) => diag.check_var(&ctx, var),
                    NodeKind::Apply(apply) => diag.check_apply(&ctx, apply),
                    NodeKind::Function(function) => diag.check_function(&ctx, function),
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
