use grustonnet_config::FormatOptions;

use crate::binding;

#[derive(rust2go::R2G, Default, Debug, Clone)]
pub struct ExtValue {
    pub name: String,
    pub value: String,
}

#[derive(rust2go::R2G, Debug)]
pub struct ASTInfo {
    pub ast_data: Vec<u8>,
    // If there is an error error_data contains the error information
    pub error_data: String,
}

#[derive(rust2go::R2G, Default, Debug, Clone)]
pub struct EvaluateParams {
    pub ext_vars: Vec<ExtValue>,
    pub ext_code: Vec<ExtValue>,
    pub jpaths: Vec<String>,
}

#[derive(rust2go::R2G, Default, Debug, Clone)]
pub struct TestData {
    pub name: String,
    pub data: Vec<u8>,
}

pub enum StringStyle {
    Double,
    Single,
    Leave,
}

pub enum CommentStyle {
    Hash,
    Slash,
    Leave,
}

#[rust2go::r2g]
pub trait ASTBridge {
    fn get_ast(filename: String) -> ASTInfo;
    fn get_ast_snippet(source_file: String, snippet: String) -> ASTInfo;
    fn get_ast_snippet_binary(source_file: String, snippet: String) -> ASTInfo;
    fn import_ast(source_file: String, filename: String, params: EvaluateParams) -> ASTInfo;
    fn evaluate_ast(ast_string: String, params: EvaluateParams) -> ASTInfo;
    fn evaluate_snippet(filename: String, snippet: String, params: EvaluateParams) -> ASTInfo;
    fn lint_snippet(filename: String, snippet: String, params: EvaluateParams) -> ASTInfo;

    fn format_snippet(filename: String, snippet: String, options: FormatOptions) -> ASTInfo;

    fn version() -> String;

    fn get_test_objects() -> Vec<TestData>;
}
