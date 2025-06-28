use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use lsp_server::ErrorCode;

use crate::server::LSPError;

pub trait ASTGenerator: Clone + Default
where
    Self::Node: ASTNode,
{
    type Node;
    fn update_ast(&self, source_file: &str, new_content: &str) -> Result<Self::Node>;
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

impl<N: ASTNode> Document<N> {
    pub fn get_ast(&self) -> Result<&N, LSPError> {
        self.ast.as_ref().ok_or(LSPError {
            error_code: ErrorCode::ParseError as i32,
            message:
                "The document was never parsed. Please fix all errors to get proper completion"
                    .to_string(),
        })
    }
}

#[derive(Default, Debug)]
pub struct Cache<G>
where
    G: ASTGenerator,
{
    documents: Arc<RwLock<HashMap<String, Document<G::Node>>>>,
    pub ast_generator: G,
}

impl<G: ASTGenerator> Cache<G> {
    pub fn new(ast_generator: G) -> Self {
        Self {
            ast_generator,
            ..Default::default()
        }
    }

    pub fn set_document(&self, name: &str, doc: Document<G::Node>) {
        self.documents.write().unwrap().insert(name.into(), doc);
    }

    pub fn remove_document(&self, name: &str) {
        self.documents.write().unwrap().remove(name);
    }

    pub fn update_content(&self, name: &str, text: &str) {
        let mut lock = self.documents.write().unwrap();
        let doc = lock.entry(name.into()).or_insert(Document::default());

        doc.filename = name.to_string();

        doc.content = text.into();

        let new_ast = self.ast_generator.update_ast(name, &doc.content);
        match new_ast {
            Ok(ast) => {
                doc.ast = Some(ast);
                doc.is_dirty = false;
            }
            Err(e) => {
                log::error!("Failed to parse ast: {e}");
                doc.is_dirty = true;
            }
        }
    }

    pub fn get_document(&self, name: &str) -> Result<Document<G::Node>, LSPError> {
        match self.documents.read().unwrap().get(name) {
            Some(val) => Ok(val.clone()),
            None => Err(LSPError {
                error_code: ErrorCode::RequestFailed as i32,
                message: "The requested document was never loaded".to_string(),
            }),
        }
    }
}
