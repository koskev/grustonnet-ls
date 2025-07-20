use language_server::{
    cache::Cache,
    server::{LSPError, LSPResponse},
    utils::UriHelper,
};
use lsp_server::ErrorCode;
use lsp_types::Uri;
use thiserror::Error;

use crate::{bridge::GenerateAST, cache::JsonnetASTGenerator};

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Unkown command {command}")]
    UnkownCommand { command: String },
    #[error("Invalid command arguments")]
    InvalidArguments,
}

impl From<CommandError> for LSPError {
    fn from(val: CommandError) -> Self {
        LSPError {
            message: val.to_string(),
            error_code: ErrorCode::ParseError as i32,
        }
    }
}

pub fn handle_command(
    cache: &Cache<JsonnetASTGenerator>,
    params: <lsp_types::request::ExecuteCommand as lsp_types::request::Request>::Params,
) -> Result<LSPResponse, LSPError> {
    #[allow(clippy::single_match)]
    match params.command.as_str() {
        "jsonnet.evalFile" => {
            if params.arguments.len() != 1 {
                return Err(CommandError::InvalidArguments.into());
            }
            let argument = &params.arguments[0];
            let eval_file_arguments: String = serde_json::from_value(argument.clone())
                .map_err(|_e| CommandError::InvalidArguments)?;
            let document = cache.get_document(&Uri::from_path(&eval_file_arguments)?)?;
            let eval_result = cache
                .ast_generator
                .jsonnet
                .evaluate_snippet(&eval_file_arguments, &document.content)?;
            return Ok(eval_result.into());
        }
        _ => {}
    }

    Err(CommandError::UnkownCommand {
        command: params.command,
    }
    .into())
}
