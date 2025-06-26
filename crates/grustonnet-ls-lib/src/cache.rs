use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use ropey::Rope;

use crate::{
    bridge::ast::{EvaluateErrorType, GenerateAST, GoJsonnet},
    node::Node,
    utils::rope::RopeHelper,
};

#[derive(Default, Debug, Clone)]
pub struct Document {
    pub content: String,
    pub ast: Node,

    pub filename: String,

    pub last_content: String,
    // If false the ast and content match. Otherwise the ast may be old
    pub is_dirty: bool,
}

impl Document {
    // BIG TODO: How to handle the modifications? AST and Editor will be out of sync
    fn get_ast(&self) -> Result<Node> {
        let mut current_content = Rope::from_str(&self.content);
        // Give up after 100 tries
        for _ in 0..100 {
            log::trace!("Document content: {}", current_content.to_string());
            let json_data = GoJsonnet::new().get_ast_snippet(&current_content.to_string());
            match json_data {
                Ok(json_data) => {
                    log::debug!("Got valid ast!");
                    return Ok(serde_json::from_str::<Node>(&json_data)?);
                }
                Err(e) => {
                    log::warn!("Error type: {}", e.error_type.variant_name());
                    match e.error_type {
                        EvaluateErrorType::ExpectedComma => {
                            // Insert comma before the given node after the first non whitespace
                            // character
                            let index = current_content.get_index(e.start);
                            let non_whitespace_idx = current_content.get_prev_non_whitespace(index);
                            current_content.insert(non_whitespace_idx + 1, ",");
                        }
                        EvaluateErrorType::ExpectedToken => {
                            let index = current_content.get_index(e.start);
                            let non_whitespace_idx = current_content.get_prev_non_whitespace(index);
                            current_content.remove(non_whitespace_idx..non_whitespace_idx + 1);
                        }
                        _ => return Err(e.into()),
                    }
                }
            }
        }
        Err(anyhow!("Unable to fix ast after 100 tries"))
    }
    pub fn update_ast(&mut self) {
        let new_ast = self.get_ast();
        match new_ast {
            Ok(node) => {
                self.ast = node;
                self.is_dirty = false;
            }
            Err(e) => {
                log::error!("Could not convert to json: {}", e);
                self.is_dirty = true;
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Cache {
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl Cache {
    pub fn set_document(&self, name: &str, doc: Document) {
        self.documents.write().unwrap().insert(name.into(), doc);
    }

    pub fn update_content(&self, name: &str, text: &str) {
        let mut lock = self.documents.write().unwrap();
        let doc = lock.entry(name.into()).or_insert(Document::default());

        if doc.filename == name.to_string() {
            doc.last_content = doc.content.clone();
        } else {
            doc.last_content = String::new();
        }
        doc.filename = name.to_string();

        doc.content = text.into();
        doc.update_ast();
    }

    pub fn get_document(&self, name: &str) -> Option<Document> {
        match self.documents.read().unwrap().get(name) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}
