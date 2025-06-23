use anyhow::anyhow;

use crate::binding;

#[derive(rust2go::R2G)]
pub struct ExtValue {
    pub name: String,
    pub value: String,
}

#[derive(rust2go::R2G)]
pub struct ASTInfo {
    pub ast_data: String,
    // If there is an error error_data contains the error information
    pub error_data: String,
}

#[derive(rust2go::R2G, Default)]
pub struct EvaluateParams {
    pub ext_vars: Vec<ExtValue>,
    pub ext_code: Vec<ExtValue>,
}

#[rust2go::r2g]
trait ASTBridge {
    fn get_ast(filename: String) -> ASTInfo;
    fn get_ast_snippet(snippet: String) -> ASTInfo;
    fn evaluate_ast(ast_string: String, params: EvaluateParams) -> ASTInfo;
    fn evaluate_snippet(filename: String, snippet: String, params: EvaluateParams) -> ASTInfo;
}

pub trait GenerateAST {
    fn get_ast(&self, filename: &str) -> anyhow::Result<String>;
    fn get_ast_snippet(&self, snippet: &str) -> anyhow::Result<String>;
    fn evaluate_ast(&self, ast_string: &str) -> anyhow::Result<String>;
    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> anyhow::Result<String>;
}

pub struct GoJsonnet {}

impl GoJsonnet {
    pub fn new() -> Self {
        Self {}
    }
}

impl GenerateAST for GoJsonnet {
    fn get_ast(&self, filename: &str) -> anyhow::Result<String> {
        let res = ASTBridgeImpl::get_ast(filename.to_string());
        if res.error_data.len() > 0 {
            return Err(anyhow!(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn get_ast_snippet(&self, snippet: &str) -> anyhow::Result<String> {
        let res = ASTBridgeImpl::get_ast_snippet(snippet.to_string());
        if res.error_data.len() > 0 {
            return Err(anyhow!(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn evaluate_ast(&self, ast_string: &str) -> anyhow::Result<String> {
        let res = ASTBridgeImpl::evaluate_ast(ast_string.to_string(), EvaluateParams::default());
        if res.error_data.len() > 0 {
            return Err(anyhow!(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> anyhow::Result<String> {
        let res = ASTBridgeImpl::evaluate_snippet(
            filename.to_string(),
            snippet.to_string(),
            EvaluateParams::default(),
        );
        if res.error_data.len() > 0 {
            return Err(anyhow!(res.error_data));
        }
        Ok(res.ast_data)
    }
}
