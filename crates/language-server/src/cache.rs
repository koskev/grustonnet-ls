use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;

pub trait ASTGenerator: Clone + Default
where
    Self::Node: ASTNode,
{
    type Node;
    fn update_ast(&self, new_content: &str) -> Result<Self::Node>;
}

pub trait ASTNode: Clone + Default {}

#[derive(Default, Debug, Clone)]
pub struct Document<N: ASTNode> {
    pub content: String,
    pub ast: Option<N>,

    pub filename: String,
    // If false the ast and content match. Otherwise the ast may be old
    pub is_dirty: bool,
}

#[derive(Default, Debug)]
pub struct Cache<G, N>
where
    G: ASTGenerator,
    N: ASTNode,
{
    documents: Arc<RwLock<HashMap<String, Document<N>>>>,
    pub ast_generator: G,
}

impl<G: ASTGenerator<Node = N>, N: ASTNode> Cache<G, N> {
    pub fn new(ast_generator: G) -> Self {
        Self {
            ast_generator,
            ..Default::default()
        }
    }

    pub fn set_document(&self, name: &str, doc: Document<N>) {
        self.documents.write().unwrap().insert(name.into(), doc);
    }

    pub fn update_content(&self, name: &str, text: &str) {
        let mut lock = self.documents.write().unwrap();
        let doc = lock.entry(name.into()).or_insert(Document::default());

        doc.filename = name.to_string();

        doc.content = text.into();

        let new_ast = self.ast_generator.update_ast(&doc.content);
        match new_ast {
            Ok(ast) => {
                doc.ast = Some(ast);
                doc.is_dirty = false;
            }
            Err(_e) => doc.is_dirty = true,
        }
    }

    pub fn get_document(&self, name: &str) -> Option<Document<N>> {
        match self.documents.read().unwrap().get(name) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}
