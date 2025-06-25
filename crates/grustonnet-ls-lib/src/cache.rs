use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    bridge::ast::{GenerateAST, GoJsonnet},
    node::Node,
};

#[derive(Default, Debug, Clone)]
pub struct Document {
    pub content: String,
    pub ast: Node,

    pub filename: String,

    // If false the ast and content match. Otherwise the ast may be old
    pub is_dirty: bool,
}

impl Document {
    pub fn update_ast(&mut self) {
        let json_data = GoJsonnet::new().get_ast_snippet(&self.content);
        match json_data {
            Ok(json_data) => {
                let node_data = serde_json::from_str::<Node>(&json_data);
                match node_data {
                    Ok(node) => {
                        self.ast = node;
                        self.is_dirty = false;
                    }
                    Err(e) => {
                        eprintln!("Could not convert to json: {}", e);
                        self.is_dirty = true;
                    }
                }
            }
            Err(e) => {
                eprintln!("Could not convert to json: {}", e);
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
