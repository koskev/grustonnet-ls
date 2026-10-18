use std::{fs, path::Path, sync::Arc};

use anyhow::{Result, anyhow};
use grustonnet_node::types::node::Node;
use language_server::cache::Cache;
use lsp_types::Uri;
use utils::uri::UriHelper;

use crate::{bridge::GenerateAST, cache::JsonnetASTGenerator};

pub struct FileImporter {
    cache: Cache<JsonnetASTGenerator>,
}

impl FileImporter {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
    pub fn import_file(&self, source_file: impl AsRef<Path>) -> Result<Arc<Node>> {
        log::debug!("Importing {:?}", source_file.as_ref().to_string_lossy());
        let content = fs::read_to_string(&source_file)?;
        let source_file_str = source_file.as_ref().to_string_lossy();

        self.import_yaml(&content)
            .or(self.import_jsonnet(&source_file_str))
    }
    fn import_yaml(&self, content: &str) -> Result<Arc<Node>> {
        let yaml_value: serde_json::Value = serde_yaml::from_str(content)?;
        let json_value = serde_json::to_string(&yaml_value)?;
        let import_ast = self
            .cache
            .ast_generator
            .jsonnet
            .get_ast_snippet_binary("", &json_value)?;
        Ok(import_ast.into())
    }
    fn import_jsonnet(&self, source_file: &str) -> Result<Arc<Node>> {
        let jpaths = self
            .cache
            .ast_generator
            .jsonnet
            .get_evaluate_params(source_file)
            .jpaths;
        let source_dir = Path::new(source_file)
            .parent()
            .ok_or(anyhow!("Unable to get parent for {}", source_file))?
            .to_str()
            .ok_or(anyhow!(
                "Unable to convert parent to str for {}",
                source_file
            ))?;
        let imported_node = jpaths
            .iter()
            .chain(std::iter::once(&source_dir.to_string()))
            .find_map(|p| {
                self.cache
                    .get_document(&Uri::from_path(format!("{}/{}", p, source_file)).ok()?)
                    .ok()?
                    .ast
            })
            .ok_or(anyhow!("Unable to find {}", source_file))?;

        Ok(imported_node)
    }
}
