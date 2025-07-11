use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    fs,
    path::Path,
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::Result;
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
        if value.starts_with("RUNTIME ERROR") {
            Self::from_runtime(value)
        } else {
            Self::from_static(value)
        }
    }
}

impl EvaluateError {
    fn unknwon_error(value: &str) -> Self {
        Self {
            filename: "unknown".to_string(),
            message: format!("unknown error: {}", value),
            ..Default::default()
        }
    }
    fn from_runtime(value: &str) -> Self {
        let uri_regex = r"(?m)RUNTIME ERROR: (?P<message>.*$)\n\s*(?P<uri>.*):";
        let location_regex = r"\(?(?P<line_start>\d+):(?P<column_start>\d+)\)?(?:-\(?(?:(?P<line_end>\d+):)?(?P<column_end>\d+)\)?)?";
        let regex = Regex::new(&format!("{uri_regex}{location_regex}+")).expect("Regex is wrong");
        let captures = regex.captures(value);

        // TODO: Support the whole stack

        let Some(captures) = captures else {
            return Self::unknwon_error(value);
        };

        let mut line_end = captures["line_start"].parse().unwrap_or_default();
        if let Some(line_end_match) = captures.name("line_end") {
            line_end = line_end_match.as_str().parse().unwrap_or_default();
        }

        Self {
            filename: captures["uri"].parse().unwrap_or_default(),
            message: captures["message"].parse().unwrap_or_default(),
            start: Location {
                line: captures["line_start"].parse().unwrap(),
                column: captures["column_start"].parse().unwrap(),
            },
            end: Location {
                line: line_end,
                column: captures["column_end"].parse().unwrap(),
            },
            ..Default::default()
        }
    }
    fn from_static(value: &str) -> Self {
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
                    // TODO: Optional Column end
                    column: captures["column_start"].parse().unwrap(),
                },
                message: captures["message"].to_string(),
                error_type: captures["message"].into(),
            },
            None => Self::unknwon_error(value),
        }
    }
}

impl Error for EvaluateError {}

#[derive(Default, Debug, Clone)]
pub struct GoJsonnet {
    root_dir: Arc<RwLock<String>>,
    config: Arc<RwLock<JsonnetConfig>>,
    params: Arc<RwLock<EvaluateParams>>,
}

fn find_upwards(cwd: &str, suffix: &str) -> HashMap<String, String> {
    // TODO: generic magic
    let mut cwd_path = Path::new(cwd);
    let mut files_found = HashMap::new();
    loop {
        let Ok(dir) = fs::read_dir(cwd_path) else {
            break;
        };
        dir.into_iter()
            .filter_map(|res| res.ok())
            .filter(|entry| match entry.file_name().into_string() {
                Ok(file_name) => {
                    //log::error!("Does {} end with {}?", file_name, suffix);
                    file_name.ends_with(suffix)
                }
                Err(_) => false,
            })
            .for_each(|found| {
                let name = found
                    .file_name()
                    .into_string()
                    .unwrap()
                    .strip_suffix(suffix)
                    .unwrap()
                    .to_string();
                if !files_found.contains_key(&name) {
                    if let Ok(content) = fs::read_to_string(found.path()) {
                        files_found.insert(name, content);
                    }
                }
            });

        match cwd_path.parent() {
            Some(parent) => cwd_path = parent,
            None => break,
        }
    }
    return files_found;
}

// TODO: performance nightmare
impl GoJsonnet {
    pub fn new(root_dir: &str) -> Self {
        Self {
            root_dir: Arc::new(RwLock::new(root_dir.to_string())),
            ..Default::default()
        }
    }

    pub fn set_root_dir(&self, dir: &str) {
        *self.root_dir.write().unwrap() = dir.to_string();
    }

    pub fn get_config(&self) -> JsonnetConfig {
        self.config.read().unwrap().clone()
    }

    pub fn set_config(&self, config: &JsonnetConfig) {
        let mut config_lock = self.config.write().unwrap();
        *config_lock = config.clone();

        // Find upwards
        let found_extcode = find_upwards(&self.root_dir.read().unwrap(), ".extcode.libsonnet");
        config_lock.ext_code.extend(
            found_extcode
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string())),
        );

        *self.params.write().unwrap() = EvaluateParams {
            ext_code: config
                .ext_code
                .iter()
                .chain(found_extcode.iter())
                .map(|(key, val)| ExtValue {
                    name: key.to_string(),
                    value: val.to_string(),
                })
                .collect(),
            ext_vars: config
                .ext_vars
                .iter()
                .map(|(key, val)| ExtValue {
                    name: key.to_string(),
                    value: val.to_string(),
                })
                .collect(),
            jpaths: config.jpaths.clone(),
        }
    }

    fn get_evaluate_params(&self, filepath: &str) -> EvaluateParams {
        let mut params = self.params.read().unwrap().clone();
        // TODO: the uri part is a mess. Just use uri everywhere?
        let uri = Uri::from_str(filepath).unwrap();
        let mut p = Path::new(uri.path().as_str());
        if p.is_file() {
            p = p.parent().unwrap()
        }
        params.jpaths.push(p.to_str().unwrap().to_string());
        params
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
