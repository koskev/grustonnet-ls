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
    pub jpaths: Vec<String>,
}

#[rust2go::r2g]
pub trait ASTBridge {
    fn get_ast(filename: String) -> ASTInfo;
    fn get_ast_snippet(source_file: String, snippet: String) -> ASTInfo;
    fn import_ast(source_file: String, filename: String, params: EvaluateParams) -> ASTInfo;
    fn evaluate_ast(ast_string: String, params: EvaluateParams) -> ASTInfo;
    fn evaluate_snippet(filename: String, snippet: String, params: EvaluateParams) -> ASTInfo;
    fn lint_snippet(filename: String, snippet: String, params: EvaluateParams) -> ASTInfo;
    fn version() -> String;
}
