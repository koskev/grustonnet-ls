use std::{path::PathBuf, time::Instant};

use anyhow::{Result, anyhow};
use itertools::Itertools;
use language_server::{
    cache::Cache,
    utils::{UriHelper, rope::RopeHelper},
};
use lsp_types::{Range, Uri};
use ropey::Rope;
#[cfg(feature = "tracing")]
use tracy_client::{secondary_frame_mark, set_thread_name, span};
use walkdir::WalkDir;

use crate::{
    cache::JsonnetASTGenerator,
    definition::DefinitionProvider,
    node::{location::Location, types::node_kind::NodeKind},
};
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
    fn get_identifier(&self, pos: Location, uri: &Uri) -> Option<String> {
        let doc = self.cache.get_document(uri).ok()?;
        let top_node = doc
            .get_ast()
            .ok()?
            .get_stack_by_position(&(pos.clone().into()))
            .peek()?;

        Some(match top_node.node_kind.as_ref() {
            NodeKind::LiteralString(s) => s.value.clone(),
            NodeKind::Var(var) => var.id.clone()?.0,
            NodeKind::Local(local) => local.get_name()?,
            NodeKind::Function(func) => func.parameters.clone()?.iter().find_map(|param| {
                if param.loc_range.in_range(&pos) {
                    Some(param.name.0.clone())
                } else {
                    None
                }
            })?,
            NodeKind::DesugaredObject(obj) => obj.get_name_at(&pos)?,
            _ => {
                log::warn!(
                    "Unhandled identifier for {}",
                    top_node.node_kind.variant_name()
                );
                return None;
            }
        })
    }

    pub fn references(
        &self,
        pos: Location,
        uri: &Uri,
        include_declaration: bool,
    ) -> Result<Option<Vec<lsp_types::Location>>> {
        let goto_provider = DefinitionProvider::new(self.cache);
        // Go to definition to find the target location
        let target_info = goto_provider.definition(uri, pos)?;
        let identifier = self
            .get_identifier(
                target_info.location.range.start.into(),
                &target_info.location.uri,
            )
            .ok_or(anyhow!("Unable to find identifier"))?;
        // Search for in all caches and files and get all potential positions
        // Get all jsonnet and libsonnet files in the search paths
        let start = Instant::now();
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
        log::info!("Getting all files took {:?}", start.elapsed());
        let start = Instant::now();
        #[cfg(feature = "tracing")]
        let zone = span!("Reference calc");
        #[cfg(feature = "tracing")]
        zone.emit_text("Calculating references");
        // Get potential positions for references
        // Open each file and searcb for the identifier and add it to the list
        let reference_locations: Vec<lsp_types::Location> = files
            .iter()
            .filter_map(|f| Uri::from_path(f.to_str().unwrap_or_default()).ok())
            .filter_map(|uri| {
                #[cfg(feature = "tracing")]
                set_thread_name!("Reference thread");
                // Check if in cache
                let content = self.cache.get_document(&uri).ok()?.content;
                // Check for name in file and get locations
                let locations: Vec<lsp_types::Location> = content
                    .match_indices(&identifier)
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
                // If the range is identical to the target we can skip the rest
                if target_info.location.range == loc.range {
                    return include_declaration;
                }
                let Ok(potential_location) =
                    goto_provider.definition(&loc.uri, loc.range.start.into())
                else {
                    return false;
                };
                // On match: Add to reference list
                potential_location.location == target_info.location
            })
            .collect();

        log::info!("Calculating references took {:?}", start.elapsed());

        if reference_locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(reference_locations))
        }
    }
}
