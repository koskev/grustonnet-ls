use std::path::{Path, PathBuf};

use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
};
use lsp_types::{CompletionItem, Position, Uri};
use walkdir::WalkDir;

use crate::cache::JsonnetASTGenerator;

pub struct ImportCompletion<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> ImportCompletion<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> Completion for ImportCompletion<'a> {
    fn complete(&self, _pos: Position, uri: &Uri) -> CompletionResult {
        let paths = self
            .cache
            .ast_generator
            .jsonnet
            .get_evaluate_params(uri.path().as_str())
            .jpaths;

        // Get all files

        let all_paths: Vec<PathBuf> = paths
            .iter()
            .flat_map(|p| {
                WalkDir::new(p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter_map(move |dir| {
                        Some(dir.into_path().strip_prefix(&p).ok()?.to_path_buf())
                    })
            })
            .collect();

        let items = all_paths
            .iter()
            .filter_map(|path| {
                Some(CompletionItem {
                    label: path.to_str()?.to_string(),
                    ..Default::default()
                })
            })
            .collect();

        Ok(lsp_types::CompletionList {
            items,
            ..Default::default()
        })
    }
}
