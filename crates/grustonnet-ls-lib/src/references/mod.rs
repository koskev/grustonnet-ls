use std::time::Instant;

use anyhow::{Result, anyhow};
use grustonnet_node::types::node_kind::NodeKind;
use jsonnet_location::Location;
use language_server::{cache::Cache, utils::rope::RopeHelper};
use lsp_types::{Range, Uri};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ropey::Rope;
#[cfg(feature = "tracing")]
use tracy_client::{set_thread_name, span};

use crate::{cache::JsonnetASTGenerator, definition::DefinitionProvider, utils};
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
    // Gets the identifier at the given position
    // Returns the string and if the identifier is limited to the current file (e.g. locals)
    fn get_identifier(&self, pos: Location, uri: &Uri) -> Option<(String, bool)> {
        let doc = self.cache.get_document(uri).ok()?;
        let mut stack = doc.get_ast().ok()?.get_stack_by_position(&pos.clone());
        if let NodeKind::Function(_func) = stack.peek()?.node_kind.as_ref() {
            // TODO: Same as in goto definition
            let _ = stack.stack.pop();
        }
        let top_node = stack.peek()?;

        Some(match top_node.node_kind.as_ref() {
            NodeKind::LiteralString(s) => (s.value.clone(), false),
            NodeKind::Var(var) => (var.id.clone()?.0, false),
            NodeKind::Local(local) => (local.get_name()?, true),
            NodeKind::Function(func) => (
                func.parameters.iter().find_map(|param| {
                    if param.loc_range.in_range(&pos) {
                        Some(param.name.0.clone())
                    } else {
                        None
                    }
                })?,
                false,
            ),
            NodeKind::DesugaredObject(obj) => (obj.get_name_at(&pos)?, false),
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
        let (identifier, is_local) = self
            .get_identifier(
                target_info.location.range.start.into(),
                &target_info.location.uri,
            )
            .ok_or(anyhow!("Unable to find identifier"))?;
        // Search for in all caches and files and get all potential positions
        // Get all jsonnet and libsonnet files in the search paths
        let start = Instant::now();
        let files = if is_local {
            vec![uri.clone()]
        } else {
            utils::files::get_all_jsonnnet_files(self.search_paths)
        };
        log::debug!("Getting all files took {:?}", start.elapsed());
        log::debug!(
            "Searching for references of {} at {} in {} files",
            identifier,
            target_info,
            files.len()
        );
        let start = Instant::now();
        #[cfg(feature = "tracing")]
        let zone = span!("Reference calc");
        #[cfg(feature = "tracing")]
        zone.emit_text("Calculating references");
        // Get potential positions for references
        // Open each file and searcb for the identifier and add it to the list
        let reference_locations: Vec<lsp_types::Location> = files
            .into_par_iter()
            .filter_map(|uri| {
                #[cfg(feature = "tracing")]
                set_thread_name!("Reference thread");
                // Check if in cache
                let content = self
                    .cache
                    .get_document_with_option(&uri, false)
                    .ok()?
                    .content;
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
                // XXX: Since goto might result in different end locations (Due to the workaround
                // with local functions), we will just compare the start position (which should be
                // enough anyways). If we land on the same position we have a reference
                let found = potential_location.location.uri == target_info.location.uri
                    && potential_location.location.range.start == target_info.location.range.start;
                log::trace!(
                    "Potential reference {} for target {}? {}",
                    potential_location,
                    target_info,
                    found
                );
                found
            })
            .collect();

        log::debug!("Calculating references took {:?}", start.elapsed());

        if reference_locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(reference_locations))
        }
    }
}
