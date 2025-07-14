use std::{
    collections::HashMap,
    fmt::Debug,
    fs,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use anyhow::{Result, anyhow};
use lsp_server::ErrorCode;
use lsp_types::Uri;

use crate::server::LSPError;

pub trait ASTGenerator: Clone + Default
where
    Self::Node: ASTNode,
{
    type Node;
    fn update_ast(&self, source_file: &str, new_content: &str) -> Result<Self::Node>;
}

pub trait ASTNode: Clone + Default + Debug {}

#[derive(Default, Debug, Clone)]
pub struct Document<G: ASTGenerator> {
    pub content: String,
    pub ast: Option<G::Node>,

    ast_generator: Arc<G>,

    pub filename: String,
    // If false the ast and content match. Otherwise the ast may be old
    pub is_dirty: bool,

    /// If the file was not opened by the lsp, we'll need to check if we need to update the content
    pub manually_loaded_at: Option<SystemTime>,
}

impl<G: ASTGenerator> Document<G> {
    pub fn get_ast(&self) -> Result<&G::Node, LSPError> {
        self.ast.as_ref().ok_or(LSPError {
            error_code: ErrorCode::ParseError as i32,
            message:
                "The document was never parsed. Please fix all errors to get proper completion"
                    .to_string(),
        })
    }

    pub fn update_content_if_needed(&mut self) -> Result<()> {
        // Get file metadata
        let modified = fs::metadata(&self.filename)?.modified()?;

        if self
            .manually_loaded_at
            .ok_or(anyhow!("Tried to update file managed by LSP"))?
            != modified
        {
            self.content = fs::read_to_string(&self.filename)?;
            self.manually_loaded_at = Some(modified);
            self.update_ast();
        }

        Ok(())
    }

    pub fn update_ast(&mut self) {
        let new_ast = self.ast_generator.update_ast(&self.filename, &self.content);
        match new_ast {
            Ok(ast) => {
                self.ast = Some(ast);
                self.is_dirty = false;
            }
            Err(e) => {
                log::error!("Failed to parse ast: {e}");
                self.is_dirty = true;
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Cache<G>
where
    G: ASTGenerator,
{
    documents: Arc<RwLock<HashMap<Uri, Document<G>>>>,
    pub ast_generator: Arc<G>,
}

impl<G: ASTGenerator> Cache<G> {
    pub fn new(ast_generator: G) -> Self {
        Self {
            ast_generator: Arc::new(ast_generator),
            ..Default::default()
        }
    }

    pub fn set_document(&self, uri: Uri, doc: Document<G>) {
        self.documents.write().unwrap().insert(uri, doc);
    }

    pub fn remove_document(&self, uri: &Uri) {
        self.documents.write().unwrap().remove(uri);
    }

    pub fn update_content(&self, uri: Uri, text: &str) {
        let mut lock = self.documents.write().unwrap();
        let doc = lock.entry(uri.clone()).or_insert(Document {
            filename: uri.as_str().into(),
            ..Default::default()
        });

        doc.content = text.into();

        doc.update_ast();
    }

    pub fn get_document(&self, uri: &Uri) -> Result<Document<G>, LSPError> {
        // TODO: lock write only after we want to manually load
        match self.documents.write().unwrap().get_mut(uri) {
            Some(val) => {
                // TODO: TEST!!
                log::debug!("Loaded from cache {}", uri.path().as_str());
                if val.manually_loaded_at.is_some() {
                    val.update_content_if_needed()?;
                }
                Ok(val.clone())
            }
            // load into cache with flag
            None => {
                log::debug!("Loading new file {}", uri.path().as_str());
                let mut doc = Document {
                    filename: uri.path().as_str().to_string(),
                    manually_loaded_at: Some(SystemTime::UNIX_EPOCH),
                    ..Default::default()
                };
                doc.update_content_if_needed()?;
                Ok(doc)
            }
        }
    }
}
