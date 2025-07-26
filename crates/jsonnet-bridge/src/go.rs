use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

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

#[derive(rust2go::R2G, Serialize, Deserialize, SmartDefault, JsonSchema, Debug, Clone)]
pub struct FormatOptions {
    // Indent is the number of spaces for each level of indenation.
    #[default = 2]
    indent: i32,
    // MaxBlankLines is the max allowed number of consecutive blank lines.
    #[default = 2]
    max_blank_lines: i32,
    #[default = 1]
    string_style: i32,
    #[default = 1]
    comment_style: i32,
    // PrettyFieldNames causes fields to only be wrapped in '' when needed.
    #[default = true]
    pretty_field_names: bool,
    // PadArrays causes arrays to be written like [ this ] instead of [this].
    #[default = false]
    pad_arrays: bool,
    // PadObjects causes arrays to be written like { this } instead of {this}.
    #[default = true]
    pad_objects: bool,
    // SortImports causes imports at the top of the file to be sorted in groups
    // by filename.
    #[default = true]
    sort_imports: bool,
    // UseImplicitPlus removes plus sign where it is not required.
    #[default = true]
    use_implicit_plus: bool,

    #[default = false]
    strip_everything: bool,
    #[default = false]
    strip_comments: bool,
    #[default = false]
    strip_all_but_comments: bool,
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
