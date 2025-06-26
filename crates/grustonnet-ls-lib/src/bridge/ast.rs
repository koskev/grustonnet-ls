use std::{
    error::Error,
    fmt::{Debug, Display},
};

use anyhow::anyhow;
use name_variant::NamedVariant;
use regex::Regex;

use crate::{binding, node::location::Location};

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
    fn get_ast(&self, filename: &str) -> Result<String, EvaluateError>;
    fn get_ast_snippet(&self, snippet: &str) -> Result<String, EvaluateError>;
    fn evaluate_ast(&self, ast_string: &str) -> Result<String, EvaluateError>;
    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError>;
}

#[derive(Debug, Default, NamedVariant)]
pub enum EvaluateErrorType {
    #[default]
    Unknown,

    ExpectedComma,
    ExpectedToken,
}

impl From<&str> for EvaluateErrorType {
    fn from(value: &str) -> Self {
        match value {
            "Expected a comma before next field" => Self::ExpectedComma,
            value if value.starts_with("Expected token IDENTIFIER but got ") => Self::ExpectedToken,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct EvaluateError {
    pub filename: String,
    pub start: Location,
    pub end: Location,

    pub message: String,

    pub error_type: EvaluateErrorType,
}

impl Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:?}-{:?} | {}",
            self.filename, self.start, self.end, self.message
        )
    }
}

impl From<String> for EvaluateError {
    fn from(value: String) -> Self {
        EvaluateError::from(value.as_str())
    }
}

impl From<&str> for EvaluateError {
    fn from(value: &str) -> Self {
        let regex = Regex::new(r"(?m)((?P<filename>.*):)?(?P<line_start>\d+):(?P<column_start>\d+)(?:-(?P<column_end>\d+))? (?P<message>.*)").unwrap();
        let captures = regex.captures(value);

        match captures {
            Some(captures) => Self {
                filename: captures
                    .name("filename")
                    .map_or(String::new(), |m| m.as_str().to_string()),
                start: Location {
                    line: captures["line_start"].parse().unwrap(),
                    column: captures["column_start"].parse().unwrap(),
                },
                end: Location {
                    line: captures["line_start"].parse().unwrap(),
                    column: captures["column_start"].parse().unwrap(),
                },
                message: captures["message"].to_string(),
                error_type: captures["message"].into(),
            },
            None => Self {
                filename: "unknown".to_string(),
                message: format!("unknown error: {}", value),
                ..Default::default()
            },
        }
    }
}

impl Error for EvaluateError {}

pub struct GoJsonnet {}

impl GoJsonnet {
    pub fn new() -> Self {
        Self {}
    }
}

impl GenerateAST for GoJsonnet {
    fn get_ast(&self, filename: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::get_ast(filename.to_string());
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn get_ast_snippet(&self, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::get_ast_snippet(snippet.to_string());
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn evaluate_ast(&self, ast_string: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::evaluate_ast(ast_string.to_string(), EvaluateParams::default());
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::evaluate_snippet(
            filename.to_string(),
            snippet.to_string(),
            EvaluateParams::default(),
        );
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }
}
