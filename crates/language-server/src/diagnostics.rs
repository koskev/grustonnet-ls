use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use crossbeam::channel::Sender;
use lsp_server::{Message, Notification};
use lsp_types::{
    PublishDiagnosticsParams, Uri,
    notification::{Notification as NotificationTrait, PublishDiagnostics},
};

use crate::utils::hashqueue::HashQueue;

pub trait DiagnosticFilter {
    fn filter_diagnostics(
        &self,
        uri: &Uri,
        results: Vec<DiagnosticsResult>,
    ) -> Vec<DiagnosticsResult>;
}

pub trait Diagnostics: Send + Sync {
    fn diagnostics(&self, uri: &Uri) -> Vec<DiagnosticsResult>;
    fn get_name(&self) -> String;
}

#[derive(Debug, Default, Clone)]
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

type CurrentDiagnostics = HashMap<Uri, HashMap<String, Vec<DiagnosticsResult>>>;
#[derive(Clone)]
pub struct DiagnosticsQueue<F>
where
    F: DiagnosticFilter + Clone,
{
    queue: Arc<Mutex<HashQueue<Uri, DiagnosticsList>>>,
    /// Contains the current active diagnostics indexed by the identifier of the lint
    pub current_diagnostics: Arc<RwLock<CurrentDiagnostics>>,
    running: Arc<RwLock<bool>>,
    sender: Sender<lsp_server::Message>,
    filter: F,
}

impl<F> DiagnosticsQueue<F>
where
    F: DiagnosticFilter + Clone,
{
    pub fn new(sender: Sender<lsp_server::Message>, filter: F) -> Self {
        Self {
            queue: Arc::new(Mutex::new(HashQueue::new())),
            running: Arc::new(RwLock::new(false)),
            sender,
            current_diagnostics: Arc::new(RwLock::new(HashMap::new())),
            filter,
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

        let mut binding = self.current_diagnostics.write().unwrap();
        let current_diag_map = binding.entry(uri.clone()).or_default();

        for diag in list {
            *current_diag_map.entry(diag.get_name()).or_default() = diag.diagnostics(&uri)
        }

        let diags = current_diag_map
            .values()
            .map(|diags| self.filter.filter_diagnostics(&uri, diags.to_vec()))
            .flat_map(|diagresults| {
                diagresults
                    .iter()
                    .map(|diag| diag.diagnostics.clone())
                    .collect::<Vec<_>>()
            })
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
