use std::{error::Error, fmt::Display};

use jsonnet_location::Location;
use language_server::server::LSPError;
use lsp_server::ErrorCode;
use name_variant::NamedVariant;
use regex::Regex;

#[derive(Debug, NamedVariant, Clone, PartialEq, Eq)]
pub enum EvaluateErrorType {
    Unknown(String),

    ExpectedComma,
    ExpectedCommaOrSemicolon,
    ExpectedToken,
    Deserialize,
}

impl Default for EvaluateErrorType {
    fn default() -> Self {
        Self::Unknown("Unknown error".into())
    }
}

impl From<&str> for EvaluateErrorType {
    fn from(value: &str) -> Self {
        match value {
            "Expected a comma before next field" => Self::ExpectedComma,
            value if value.starts_with("Expected , or ; but got ") => {
                Self::ExpectedCommaOrSemicolon
            }
            value if value.starts_with("Expected token IDENTIFIER but got ") => Self::ExpectedToken,
            _ => Self::Unknown(value.to_string()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

impl From<EvaluateError> for LSPError {
    fn from(val: EvaluateError) -> Self {
        LSPError {
            message: val.to_string(),
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
