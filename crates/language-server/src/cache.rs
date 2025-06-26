use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;

#[derive(Default, Debug, Clone)]
pub struct NullAST;

impl ASTGenerator for NullAST {
    fn update_ast(&mut self, _: &str) -> Result<()> {
        Ok(())
    }
}

pub trait ASTGenerator: Clone + Default {
    fn update_ast(&mut self, new_content: &str) -> Result<()>;
}

#[derive(Default, Debug, Clone)]
pub struct Document<G: ASTGenerator> {
    pub content: String,
    pub ast_generator: G,

    pub filename: String,
    // If false the ast and content match. Otherwise the ast may be old
    pub is_dirty: bool,
}

#[derive(Default, Debug)]
pub struct Cache<G: ASTGenerator> {
    documents: Arc<RwLock<HashMap<String, Document<G>>>>,
}

impl<G: ASTGenerator> Cache<G> {
    pub fn set_document(&self, name: &str, doc: Document<G>) {
        self.documents.write().unwrap().insert(name.into(), doc);
    }

    pub fn update_content(&self, name: &str, text: &str) {
        let mut lock = self.documents.write().unwrap();
        let doc = lock.entry(name.into()).or_insert(Document::default());

        doc.filename = name.to_string();

        doc.content = text.into();
        doc.is_dirty = doc.ast_generator.update_ast(&doc.content).is_err();
    }

    pub fn get_document(&self, name: &str) -> Option<Document<G>> {
        match self.documents.read().unwrap().get(name) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}
