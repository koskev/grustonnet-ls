use std::collections::HashMap;

use crate::binding;

#[derive(rust2go::R2G)]
pub struct ExtValue {
    pub name: String,
    pub value: String,
}

#[rust2go::r2g]
pub trait GenerateAST {
    fn get_ast(filename: String) -> String;
    fn get_ast_snippet(snippet: String) -> String;
    fn evaluate_ast(ast_string: String, ext_vars: Vec<ExtValue>, ext_code: Vec<ExtValue>)
    -> String;
}
