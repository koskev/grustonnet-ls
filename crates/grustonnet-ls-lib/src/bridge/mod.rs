use std::{
    error::Error,
    fmt::{Debug, Display},
    path::Path,
    str::FromStr,
    sync::{Arc, RwLock},
};

use jsonnet_bridge::go::{ASTBridge, ASTBridgeImpl, EvaluateParams, ExtValue, FormatOptions};
use language_server::server::LSPError;
use lsp_server::ErrorCode;
use lsp_types::Uri;
use name_variant::NamedVariant;
use regex::Regex;

use crate::{node::location::Location, server::config::JsonnetConfig};

pub trait GenerateAST {
    fn get_ast(&self, filename: &str) -> Result<String, EvaluateError>;
    fn get_ast_snippet(&self, source_file: &str, snippet: &str) -> Result<String, EvaluateError>;
    fn import_ast(&self, source_file: &str, filename: &str) -> Result<String, EvaluateError>;
    fn evaluate_ast(&self, ast_string: &str, source_file: &str) -> Result<String, EvaluateError>;
    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError>;
    fn lint_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError>;

    fn format_snippet(
        &self,
        filename: &str,
        snippet: &str,
        options: &FormatOptions,
    ) -> Result<String, EvaluateError>;
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

impl Into<LSPError> for EvaluateError {
    fn into(self) -> LSPError {
        LSPError {
            message: self.to_string(),
            error_code: ErrorCode::ParseError as i32,
        }
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

#[derive(Default, Debug, Clone)]
pub struct GoJsonnet {
    pub config: Arc<RwLock<JsonnetConfig>>,
}

impl GoJsonnet {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    // TODO: this is a performance nightmare
    fn get_evaluate_params(&self, filepath: &str) -> EvaluateParams {
        // TODO: the uri part is a mess. Just use uri everywhere?
        let uri = Uri::from_str(filepath).unwrap();
        let mut p = Path::new(uri.path().as_str());
        if p.is_file() {
            p = p.parent().unwrap()
        }
        let mut jpaths = vec![p.to_str().unwrap().to_string()];
        jpaths.extend(self.config.read().unwrap().jpaths.clone());
        EvaluateParams {
            ext_code: self
                .config
                .read()
                .unwrap()
                .ext_code
                .iter()
                .map(|(key, val)| ExtValue {
                    name: key.to_string(),
                    value: val.to_string(),
                })
                .collect(),
            ext_vars: self
                .config
                .read()
                .unwrap()
                .ext_vars
                .iter()
                .map(|(key, val)| ExtValue {
                    name: key.to_string(),
                    value: val.to_string(),
                })
                .collect(),
            jpaths,
        }
    }
}

impl GenerateAST for GoJsonnet {
    fn import_ast(&self, source_file: &str, filename: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::import_ast(
            source_file.to_string(),
            filename.to_string(),
            self.get_evaluate_params(source_file),
        );
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }
    fn get_ast(&self, filename: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::get_ast(filename.to_string());
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn get_ast_snippet(&self, source_file: &str, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::get_ast_snippet(source_file.to_string(), snippet.to_string());
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn evaluate_ast(&self, ast_string: &str, source_file: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::evaluate_ast(
            ast_string.to_string(),
            self.get_evaluate_params(source_file),
        );
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::evaluate_snippet(
            filename.to_string(),
            snippet.to_string(),
            self.get_evaluate_params(filename),
        );
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn lint_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::lint_snippet(
            filename.to_string(),
            snippet.to_string(),
            self.get_evaluate_params(filename),
        );
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn format_snippet(
        &self,
        filename: &str,
        snippet: &str,
        options: &FormatOptions,
    ) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::format_snippet(
            filename.to_string(),
            snippet.to_string(),
            options.clone(),
        );
        if res.error_data.len() > 0 {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }
}
