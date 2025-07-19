use std::{
    collections::{HashMap, hash_map::Entry},
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::anyhow;
use language_server::{
    cache::Cache,
    completion::{Completion, CompletionResult},
    utils::UriHelper,
};
use lsp_types::{CompletionItem, CompletionItemKind, Position, Uri};
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

struct ImportFolder {
    // Easy way out :D
    jpath: Rc<String>,
    file: PathBuf,
}

impl ImportFolder {
    fn new(jpath: Rc<String>, file: PathBuf) -> Self {
        Self { jpath, file }
    }
}

impl TryInto<Uri> for &ImportFolder {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Uri, Self::Error> {
        let full_path = Path::new(self.jpath.as_str()).join(self.file.clone());
        Uri::from_path(
            full_path
                .to_str()
                .ok_or(anyhow!("Could not convert path to str"))?,
        )
    }
}

/// Complete all possible import from all configured JPaths
// TODO: Get the typed string and support completion after "/". The editor might not consider the
// parts before the slash
impl<'a> Completion for ImportCompletion<'a> {
    fn complete(&self, _pos: Position, uri: &Uri) -> CompletionResult {
        let extensions = ["jsonnet", "libsonnet", "json"];
        // Allow jsonpkg imports
        let prefixes = ["@"];
        let paths = self
            .cache
            .ast_generator
            .jsonnet
            .get_evaluate_params(uri.path().as_str())
            .jpaths;

        // Get all files
        let all_paths: Vec<ImportFolder> = paths
            .into_iter()
            .flat_map(|p| {
                let path_rc = Rc::new(p.clone());
                WalkDir::new(p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter(|e| {
                        extensions.contains(
                            &e.path()
                                .extension()
                                .and_then(OsStr::to_str)
                                .unwrap_or_default(),
                        ) || prefixes.iter().any(|p| e.path().starts_with(p))
                    })
                    .filter_map(move |dir| {
                        Some(ImportFolder::new(
                            path_rc.clone(),
                            dir.into_path()
                                .strip_prefix(path_rc.as_str())
                                .ok()?
                                .to_path_buf(),
                        ))
                    })
            })
            .collect();

        let mut filtered_paths = HashMap::new();
        for path in &all_paths {
            match filtered_paths.entry(path.file.clone()) {
                Entry::Vacant(e) => {
                    e.insert(path);
                }
                Entry::Occupied(mut e) => {
                    if e.get().jpath.len() > path.jpath.len() {
                        e.insert(path);
                    }
                }
            }
        }

        let items = filtered_paths
            .into_values()
            .filter_map(|path| {
                let uri: Uri = path.try_into().ok()?;
                Some(CompletionItem {
                    label: path.file.to_str()?.to_string().clone(),
                    detail: Some(
                        self.cache
                            .get_document_with_option(&uri, false)
                            .ok()?
                            .content,
                    ),
                    kind: Some(CompletionItemKind::FILE),
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
