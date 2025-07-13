use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use itertools::Itertools;
use language_server::{
    cache::Cache,
    utils::{UriHelper, rope::RopeHelper},
};
use lsp_types::{Range, Uri};
use ropey::Rope;
use walkdir::WalkDir;

use crate::{cache::JsonnetASTGenerator, definition::DefinitionProvider, node::location::Location};
pub struct ReferenceProvider<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub search_paths: &'a [String],
}

impl<'a> ReferenceProvider<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>, search_paths: &'a [String]) -> Self {
        Self {
            cache,
            search_paths,
        }
    }
}

impl<'a> ReferenceProvider<'a> {
    pub fn references(&self, pos: Location, uri: &Uri) -> Result<Option<Vec<lsp_types::Location>>> {
        let doc = self.cache.get_document(uri)?;
        let document_stack = doc.get_ast()?.get_stack_by_position(&(pos.into()));
        let top_node = document_stack.peek().ok_or(anyhow!("No node in stack"))?;
        let goto_provider = DefinitionProvider::new(self.cache);
        // Go to definition to find the target location
        let target_info = goto_provider.definition(
            &Uri::from_path(&top_node.node_base.loc_range.file_name).unwrap(),
            top_node.node_base.loc_range.begin,
        )?;
        // Search for in all caches and files and get all potential positions
        // Get all jsonnet and libsonnet files in the search paths
        let files: Vec<PathBuf> = self
            .search_paths
            .iter()
            .flat_map(|p| {
                WalkDir::new(p)
                    .into_iter()
                    .filter_map(|r| r.ok())
                    .filter(|f| {
                        f.path().is_file()
                            && f.path()
                                .extension()
                                .map(|e| e == "jsonnet" || e == "libsonnet")
                                .unwrap_or(false)
                    })
                    .map(|f| f.into_path())
            })
            .unique()
            .collect();
        // Get potential positions for references
        // Open each file and searcb for the identifier and add it to the list
        let reference_locations: Vec<lsp_types::Location> = files
            .iter()
            .filter_map(|f| Uri::from_path(f.to_str().unwrap_or_default()).ok())
            .filter_map(|uri| {
                // Check if in cache
                let content = if let Ok(doc) = self.cache.get_document(&uri) {
                    Some(doc.content)
                } else {
                    // If not load file
                    fs::read_to_string(uri.path().as_str()).ok()
                }?;
                // Check for name in file and get locations
                let locations: Vec<lsp_types::Location> = content
                    .match_indices(&target_info.name)
                    .filter_map(|(index, val)| {
                        let rope = Rope::from_str(&content);
                        Some(lsp_types::Location {
                            uri: uri.clone(),
                            range: Range {
                                start: rope.get_location(index)?,
                                end: rope.get_location(index + val.len())?,
                            },
                        })
                    })
                    .collect();

                Some(locations)
            })
            .flatten()
            // Execute a goto and compare it's position with the target position
            .filter(|loc| {
                let Ok(target_location) =
                    goto_provider.definition(&loc.uri, loc.range.start.into())
                else {
                    return false;
                };
                // On match: Add to reference list
                target_location.location == target_info.location
            })
            .collect();

        if reference_locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(reference_locations))
        }
    }
}
