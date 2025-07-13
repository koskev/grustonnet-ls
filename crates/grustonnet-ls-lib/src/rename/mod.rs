use std::collections::HashMap;

use anyhow::{Result, anyhow};
use language_server::cache::Cache;
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};

use crate::{cache::JsonnetASTGenerator, references::ReferenceProvider};

pub struct RenameProvider<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> RenameProvider<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl<'a> RenameProvider<'a> {
    pub fn rename(&self, params: RenameParams, search_paths: &[String]) -> Result<WorkspaceEdit> {
        let reference_provider = ReferenceProvider::new(self.cache, search_paths);

        let references = reference_provider
            .references(
                params.text_document_position.position.into(),
                &params.text_document_position.text_document.uri,
            )?
            .ok_or(anyhow!("No references found"))?;

        // Rename all references
        let edits = references.into_iter().fold(HashMap::new(), |mut acc, loc| {
            acc.entry(loc.uri.clone()).or_insert(vec![]).push(TextEdit {
                range: loc.range,
                new_text: params.new_name.clone(),
            });
            acc
        });

        let workspace_edit = WorkspaceEdit {
            changes: Some(edits),
            ..Default::default()
        };

        Ok(workspace_edit)
    }
}
