use std::{
    collections::HashMap,
    fs,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Result, anyhow};
use bincode::{
    Decode, Encode,
    de::{self},
};
use crossbeam::channel::Sender;
use grustonnet_node::types::{Identifier, node::Node};
use jsonnet_bridge::go::{ASTInfo, DebuggerBridge, DebuggerBridgeImpl, EvaluateParams, ExtValue};
use jsonnet_location::LocationRange;
use rust_dap::{
    server::{DAPConnection, DAPError, DAPResponse, DAPServer},
    types::{
        events::{self, Event},
        messages::{EventMessage, MessageType},
        requests::{
            Continue, Evaluate, Next, Request, Scopes, StackTrace, StepIn, Threads, Variables,
        },
        types::{
            Breakpoint, Capabilities, ConfigurationDoneArguments, EvaluateResponse,
            InitializeRequestArguments, LaunchRequestArguments, Scope, ScopesResponse,
            SetBreakpointsArguments, SetBreakpointsResponse, Source, StackFrame,
            StackTraceResponse, StoppedEvent, StoppedEventReason, Thread, ThreadsResponse,
            Variable, VariablesResponse,
        },
    },
};
use serde::{Deserialize, Serialize};
use utils::RwLockPanic;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", default)]
struct DebugEventExit {
    output: String,
    error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", default)]
struct DebugEventStop {
    reason: i32,
    breakpoint: String,
    current: Node,
    last_evaluation: String,
    error: String,
    // efmt is used to format the error (if any). Built by the vm so we need to
    // keep a reference in the event
    //efmt ErrorFormatter
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
enum DebugEvent {
    Stop(DebugEventStop),
    Exit(DebugEventExit),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", default)]
struct DebugStackElement {
    name: String,
    loc: LocationRange,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", default)]
struct DebugStackTrace(Vec<DebugStackElement>);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchConfig {
    program: String,
    j_paths: Vec<String>,
    ext_vars: HashMap<String, String>,
    ext_code: HashMap<String, String>,
}

pub struct JsonnetDAPServer {
    pub connection: DAPConnection,
    pub launch_config: Arc<RwLock<LaunchConfig>>,
    pub init_args: Arc<RwLock<Option<InitializeRequestArguments>>>,
}

impl JsonnetDAPServer {
    pub fn new(connection: DAPConnection, running: Arc<AtomicBool>) -> Self {
        spawn_debugger_thread(connection.sender.clone(), running.clone());
        JsonnetDAPServer {
            connection,
            launch_config: Arc::default(),
            init_args: Arc::default(),
        }
    }
}

fn spawn_debugger_thread(server_tx: Sender<MessageType>, running: Arc<AtomicBool>) {
    rayon::spawn(move || {
        while running.load(Ordering::Relaxed) {
            let _ = (|| -> Result<()> {
                log::debug!("Waiting for Jsonnet Debugger Event");
                let res = DebuggerBridgeImpl::wait_for_event();
                let event_data: DebugEvent = decode(&res)?;
                let event = match event_data {
                    DebugEvent::Stop(stop_event) => {
                        let reason = match stop_event.reason {
                            0 => StoppedEventReason::Step,
                            1 => StoppedEventReason::Breakpoint,
                            2 => StoppedEventReason::Exception,
                            _ => StoppedEventReason::Unknown,
                        };
                        EventMessage {
                            event: events::Stopped::EVENT.into(),
                            body: Some(serde_json::to_value(StoppedEvent {
                                description: None,
                                thread_id: Some(1),
                                preserve_focus_hint: None,
                                text: if stop_event.error.is_empty() {
                                    None
                                } else {
                                    Some(stop_event.error.clone())
                                },
                                all_threads_stopped: Some(true),
                                hit_breakpoint_ids: None,
                                reason,
                            })?),
                        }
                    }
                    DebugEvent::Exit(_exit_event) => {
                        running.store(false, Ordering::Relaxed);
                        EventMessage {
                            event: events::Terminated::EVENT.into(),
                            body: None,
                        }
                    }
                };
                server_tx.send(MessageType::Event(event))?;
                Ok(())
            })();
        }
    });
}

impl DAPServer for JsonnetDAPServer {
    fn connection(&self) -> &DAPConnection {
        &self.connection
    }

    fn initialize(&self, args: InitializeRequestArguments) -> Result<DAPResponse, DAPError> {
        log::debug!("INIT");
        *self.init_args.write_or_panic() = Some(args);

        Ok(Capabilities {
            supports_configuration_done_request: Some(true),
            ..Default::default()
        }
        .into())
    }

    fn launch(&self, args: LaunchRequestArguments) -> Result<DAPResponse, DAPError> {
        let new_config: LaunchConfig =
            serde_json::from_value(args.raw).expect("Decoding launch values");
        *self.launch_config.write_or_panic() = new_config.clone();
        Ok(().into())
    }

    fn set_breakpoints(&self, args: SetBreakpointsArguments) -> Result<DAPResponse, DAPError> {
        log::debug!("SET BREAKPOINT");

        let filename = args
            .source
            .path
            .clone()
            .ok_or(anyhow!("Invalid filename"))?;
        DebuggerBridgeImpl::clear_breakpoints(filename.clone());
        let breakpoints = args
            .breakpoints
            .unwrap_or_default()
            .iter()
            .filter_map(|breakpoint| {
                let ret_val = DebuggerBridgeImpl::add_breakpoint(
                    filename.clone(),
                    breakpoint.line as i64,
                    //breakpoint.column.unwrap_or_default(),
                    -1,
                );
                if !ret_val.error.is_empty() {
                    None
                } else {
                    Some(Breakpoint {
                        line: Some(breakpoint.line),
                        verified: true,
                        ..Default::default()
                    })
                }
            })
            .collect();

        Ok(SetBreakpointsResponse { breakpoints }.into())
    }
    fn configuration_done(
        &self,
        _args: ConfigurationDoneArguments,
    ) -> Result<DAPResponse, DAPError> {
        let filename = self.launch_config.read_or_panic().program.clone();
        let content = fs::read_to_string(&filename).expect("Reading file");
        // XXX: "Launch" will also start the debugger. So we can only start it after the
        // configuration is done
        let mut jpaths = vec![".".into()];
        jpaths.extend(self.launch_config.read_or_panic().j_paths.clone());
        DebuggerBridgeImpl::launch(
            filename,
            content,
            EvaluateParams {
                jpaths,
                ext_vars: self
                    .launch_config
                    .read_or_panic()
                    .ext_vars
                    .clone()
                    .into_iter()
                    .map(|(name, value)| ExtValue { name, value })
                    .collect(),
                ext_code: self
                    .launch_config
                    .read_or_panic()
                    .ext_code
                    .clone()
                    .into_iter()
                    .map(|(name, value)| ExtValue { name, value })
                    .collect(),
            },
        );
        log::info!("Launch done");

        Ok(().into())
    }

    fn get_threads(&self, _args: <Threads as Request>::Arguments) -> Result<DAPResponse, DAPError> {
        Ok(ThreadsResponse {
            threads: vec![Thread {
                id: 1,
                name: "jsonnet".into(),
            }],
        }
        .into())
    }

    fn get_stack_trace(
        &self,
        _args: <StackTrace as Request>::Arguments,
    ) -> Result<DAPResponse, DAPError> {
        let res = DebuggerBridgeImpl::get_stack_trace();
        let (data, _) = bincode::decode_from_slice::<DebugStackTrace, _>(
            &res.ast_data,
            bincode::config::legacy(),
        )
        .expect("Decoding stack trace");
        let frames: Vec<_> = data
            .0
            .iter()
            .rev()
            .enumerate()
            .map(|(i, frame)| {
                let filename = frame
                    .loc
                    .file
                    .clone()
                    .unwrap_or_default()
                    .diagnostic_file_name;
                let path = std::path::absolute(&filename)
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let path = if path.is_empty() { None } else { Some(path) };
                let mut debug_frame = StackFrame {
                    id: i as u64,
                    name: frame.name.clone(),
                    ..Default::default()
                };

                if path.is_some() {
                    debug_frame.line = frame.loc.begin.line as u64;
                    debug_frame.column = frame.loc.begin.column as u64;
                    debug_frame.end_line = Some(frame.loc.end.line as u64);
                    debug_frame.end_column = Some(frame.loc.end.column as u64);
                    debug_frame.source = Some(Source {
                        name: Some(filename.clone()),
                        path,
                        ..Default::default()
                    });
                }
                debug_frame
            })
            .collect();
        Ok(StackTraceResponse {
            total_frames: Some(frames.len() as u64),
            stack_frames: frames,
        }
        .into())
    }

    fn continue_debugger(
        &self,
        _args: <Continue as Request>::Arguments,
    ) -> Result<DAPResponse, DAPError> {
        DebuggerBridgeImpl::continue_debugger();
        Ok(().into())
    }
    fn step_in(&self, _args: <StepIn as Request>::Arguments) -> Result<DAPResponse, DAPError> {
        DebuggerBridgeImpl::step();
        Ok(().into())
    }
    fn next(&self, _args: <Next as Request>::Arguments) -> Result<DAPResponse, DAPError> {
        DebuggerBridgeImpl::step_over();
        Ok(().into())
    }

    fn scopes(&self, _args: <Scopes as Request>::Arguments) -> Result<DAPResponse, DAPError> {
        Ok(ScopesResponse {
            scopes: vec![Scope {
                name: "Local".into(),
                variables_reference: 1,
                ..Default::default()
            }],
        }
        .into())
    }

    fn variables(&self, _args: <Variables as Request>::Arguments) -> Result<DAPResponse, DAPError> {
        let info = DebuggerBridgeImpl::list_vars();
        let mut decoded: Vec<Identifier> = decode(&info)?;
        let self_identifier = Identifier("self".into());
        if !decoded.contains(&self_identifier) {
            decoded.push(self_identifier);
        }

        let dap_vars = decoded
            .iter()
            .filter_map(|identifier| {
                let value = DebuggerBridgeImpl::lookup_value(identifier.0.clone())
                    .get_string()
                    .ok()?;

                Some(Variable {
                    name: identifier.0.clone(),
                    value,
                    ..Default::default()
                })
            })
            .collect();

        Ok(VariablesResponse {
            variables: dap_vars,
        }
        .into())
    }

    fn evaluate(&self, args: <Evaluate as Request>::Arguments) -> Result<DAPResponse, DAPError> {
        let info = DebuggerBridgeImpl::evaluate(args.expression).get_string()?;

        let resp = EvaluateResponse {
            result: info.clone(),
            ..Default::default()
        };

        Ok(resp.into())
    }
}

fn decode<D>(input: &ASTInfo) -> Result<D>
where
    D: de::Decode<()>,
{
    if !input.error_data.is_empty() {
        Err(anyhow!(input.error_data.clone()))
    } else {
        let (data, _) = bincode::decode_from_slice(&input.ast_data, bincode::config::legacy())?;
        Ok(data)
    }
}
