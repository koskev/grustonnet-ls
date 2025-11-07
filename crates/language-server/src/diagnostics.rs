use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use crossbeam::channel::Sender;
use lsp_server::{Message, Notification};
use lsp_types::{
    Diagnostic, PublishDiagnosticsParams, Uri,
    notification::{Notification as NotificationTrait, PublishDiagnostics},
};

use crate::utils::hashqueue::HashQueue;

pub trait Diagnostics: Send + Sync {
    fn diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult>;
}

#[derive(Debug, Default)]
pub struct DiagnosticsResult {
    pub diagnostics: lsp_types::Diagnostic,
    pub code_actions: Vec<lsp_types::CodeAction>,
}

impl From<lsp_types::Diagnostic> for DiagnosticsResult {
    fn from(value: lsp_types::Diagnostic) -> Self {
        Self {
            diagnostics: value,
            ..Default::default()
        }
    }
}

pub type DiagnosticsList = Vec<Box<dyn Diagnostics>>;

#[derive(Clone)]
pub struct DiagnosticsQueue {
    queue: Arc<Mutex<HashQueue<Uri, DiagnosticsList>>>,
    running: Arc<RwLock<bool>>,
    sender: Sender<lsp_server::Message>,
}

impl DiagnosticsQueue {
    pub fn new(sender: Sender<lsp_server::Message>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(HashQueue::new())),
            running: Arc::new(RwLock::new(false)),
            sender,
        }
    }

    pub fn queue(&self, uri: Uri, diagnostics: DiagnosticsList) {
        self.queue.lock().unwrap().push(uri, diagnostics);
    }

    pub fn process_queue(&self) {
        let Some((uri, list)) = self.queue.lock().unwrap().pop() else {
            return;
        };
        log::trace!("Processing diagnostics for {:?}", uri);
        let diags: Vec<Diagnostic> = list
            .iter()
            .flat_map(|d| d.diagnostics(&uri))
            .map(|d| d.diagnostics)
            .collect();
        // Always send the notification to clear old diagnostic messages
        self.sender
            .send(Message::Notification(Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params: serde_json::to_value(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics: diags,
                    version: None,
                })
                .unwrap(),
            }))
            .unwrap();
    }

    pub fn stop(&self) {
        *self.running.write().unwrap() = false;
    }

    pub fn run(&self) {
        *self.running.write().unwrap() = true;
        while *self.running.read().unwrap() {
            self.process_queue();
            // TODO: Blocking wait until there is data?
            thread::sleep(Duration::from_millis(10));
        }
    }
}
