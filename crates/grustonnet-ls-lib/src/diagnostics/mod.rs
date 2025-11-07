use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use language_server::{diagnostics::Diagnostics, utils::hashqueue::HashQueue};
use lsp_types::{Diagnostic, Uri};

pub mod eval;
pub mod go_lint;
pub mod lint;

type DiagnosticsList = Vec<Box<dyn Diagnostics>>;

pub struct DiagnosticsQueue {
    queue: Arc<Mutex<HashQueue<Uri, DiagnosticsList>>>,
    running: AtomicBool,
}

impl DiagnosticsQueue {
    pub fn queue_document(&self, uri: Uri, diagnostics: DiagnosticsList) {
        self.queue.lock().unwrap().push(uri, diagnostics);
    }

    pub fn process_queue(&self) -> Vec<Diagnostic> {
        let Some((uri, list)) = self.queue.lock().unwrap().pop() else {
            return vec![];
        };
        list.iter()
            .flat_map(|d| d.diagnostics(&uri))
            .map(|d| d.diagnostics)
            .collect()
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn run(&self) {
        while self.running.load(Ordering::Relaxed) {
            self.process_queue();
            // TODO: Blocking wait until there is data?
            sleep(Duration::from_secs(1));
        }
    }
}
