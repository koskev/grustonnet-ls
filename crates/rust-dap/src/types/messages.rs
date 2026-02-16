use std::io::{self, BufRead};

use lsp_server::ExtractError;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::types::types::Capabilities;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageBase {
    pub seq: u64,

    #[serde(flatten)]
    pub message: MessageType,
}

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

macro_rules! invalid_data {
    ($($tt:tt)*) => (invalid_data(format!($($tt)*)))
}

fn read_msg_text(inp: &mut dyn BufRead) -> std::io::Result<Option<String>> {
    let mut size = None;
    let mut buf = String::new();
    loop {
        buf.clear();
        if inp.read_line(&mut buf)? == 0 {
            return Ok(None);
        }
        if !buf.ends_with("\r\n") {
            return Err(invalid_data!("malformed header: {:?}", buf));
        }
        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }
        let mut parts = buf.splitn(2, ": ");
        let header_name = parts.next().expect("Getting next parts");
        let header_value = parts
            .next()
            .ok_or_else(|| invalid_data!("malformed header: {:?}", buf))?;
        if header_name.eq_ignore_ascii_case("Content-Length") {
            size = Some(header_value.parse::<usize>().map_err(invalid_data)?);
        }
    }
    let size: usize = size.ok_or_else(|| invalid_data!("no Content-Length"))?;
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    inp.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf).map_err(invalid_data)?;
    log::debug!("< {buf}");
    Ok(Some(buf))
}

impl MessageBase {
    pub fn read(reader: &mut dyn BufRead) -> std::io::Result<Option<MessageBase>> {
        let text = match read_msg_text(reader)? {
            None => return Ok(None),
            Some(text) => text,
        };

        let msg = match serde_json::from_str(&text) {
            Ok(msg) => msg,
            Err(e) => {
                return Err(invalid_data!("malformed DAP payload: {:?}", e));
            }
        };

        Ok(Some(msg))
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MessageType {
    Request(RequestMessage),
    Response(ResponseMessage),
    Event(EventMessage),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestMessage {
    pub command: String,
    // TODO: Rewrite generator to use enums instead of raw values
    pub arguments: Option<serde_json::Value>,
}

impl RequestMessage {
    pub fn extract<P: DeserializeOwned>(
        self,
        method: &str,
    ) -> Result<P, ExtractError<RequestMessage>> {
        if self.command != method {
            return Err(ExtractError::MethodMismatch(self));
        }
        match serde_json::from_value(self.arguments.unwrap_or_default()) {
            Ok(params) => Ok(params),
            Err(error) => Err(ExtractError::JsonError {
                method: self.command,
                error,
            }),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EventMessage {
    pub event: String,
    pub body: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseMessage {
    pub request_seq: u64,
    pub success: bool,
    pub command: String,
    pub message: Option<String>,
    pub body: Option<serde_json::Value>,
}

#[allow(clippy::derivable_impls)]
impl Default for Capabilities {
    fn default() -> Self {
        Self {
            supports_data_breakpoint_bytes: None,
            supports_ansistyling: None,
            supports_configuration_done_request: None,
            supports_function_breakpoints: None,
            supports_conditional_breakpoints: None,
            supports_hit_conditional_breakpoints: None,
            supports_evaluate_for_hovers: None,
            exception_breakpoint_filters: None,
            supports_step_back: None,
            supports_set_variable: None,
            supports_restart_frame: None,
            supports_goto_targets_request: None,
            supports_step_in_targets_request: None,
            supports_completions_request: None,
            completion_trigger_characters: None,
            supports_modules_request: None,
            additional_module_columns: None,
            supported_checksum_algorithms: None,
            supports_restart_request: None,
            supports_exception_options: None,
            supports_value_formatting_options: None,
            supports_exception_info_request: None,
            support_terminate_debuggee: None,
            support_suspend_debuggee: None,
            supports_delayed_stack_trace_loading: None,
            supports_loaded_sources_request: None,
            supports_log_points: None,
            supports_terminate_threads_request: None,
            supports_set_expression: None,
            supports_terminate_request: None,
            supports_data_breakpoints: None,
            supports_read_memory_request: None,
            supports_write_memory_request: None,
            supports_disassemble_request: None,
            supports_cancel_request: None,
            supports_breakpoint_locations_request: None,
            supports_clipboard_context: None,
            supports_stepping_granularity: None,
            supports_instruction_breakpoints: None,
            supports_exception_filter_options: None,
            supports_single_thread_execution_requests: None,
            breakpoint_modes: None,
        }
    }
}
