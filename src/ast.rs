use serde::Serialize;

use crate::binding;

pub struct LiteralBool {
    pub val: bool,
}

pub struct BaseNode {}

pub trait ASTNode {}

#[rust2go::r2g]
pub trait GenerateAST {
    fn get_ast(filename: String) -> String;
    fn get_ast_snippet(snippet: String) -> String;
}
