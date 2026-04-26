use grustonnet_node::types::node_kind::NodeKind;
use jsonnet_location::{FileRange, Location, Range};
use language_server::cache::Cache;
use lsp_types::Uri;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ropey::Rope;
#[cfg(feature = "tracing")]
use tracy_client::{set_thread_name, span};
use utils::rope::RopeHelper;

use crate::{cache::JsonnetASTGenerator, references::ReferenceProvider};

pub struct IdentifierReferences {
    pub cache: Cache<JsonnetASTGenerator>,
}

impl ReferenceProvider for IdentifierReferences {
    fn is_valid(&self, location: FileRange) -> bool {
        let identifier_option = self.get_identifier(location.range.begin, &location.uri);
        identifier_option.is_some()
    }

    fn local_only(&self, location: FileRange) -> bool {
        let identifier_option = self.get_identifier(location.range.begin, &location.uri);

        match identifier_option {
            Some(option) => option.1,
            None => true,
        }
    }
    fn generate_potential_locations(
        &self,
        target_info: &crate::definition::DefinitionInfo,
        files: &[lsp_types::Uri],
    ) -> Option<Vec<FileRange>> {
        let (identifier, _is_local) = self.get_identifier(
            target_info.location.range.begin.clone(),
            &target_info.location.uri,
        )?;
        // Search for in all caches and files and get all potential positions
        // Get all jsonnet and libsonnet files in the search paths
        #[cfg(feature = "tracing")]
        let zone = span!("Reference calc");
        #[cfg(feature = "tracing")]
        zone.emit_text("Calculating references");
        // Get potential positions for references
        // Open each file and search for the identifier and add it to the list
        Some(
            files
                .into_par_iter()
                .filter_map(move |uri| {
                    #[cfg(feature = "tracing")]
                    set_thread_name!("Reference thread");
                    // Check if in cache
                    let content = self
                        .cache
                        .get_document_with_option(uri, false)
                        .ok()?
                        .content;
                    // Check for name in file and get locations
                    let locations: Vec<FileRange> = content
                        .match_indices(&identifier)
                        .filter_map(|(index, val)| {
                            let rope = Rope::from_str(&content);
                            Some(FileRange {
                                uri: uri.clone(),
                                range: Range {
                                    // TODO: Properly handle utf16 etc.
                                    begin: rope.get_location_from_byte(index)?.into(),
                                    end: rope.get_location_from_byte(index + val.len())?.into(),
                                },
                            })
                        })
                        .collect();

                    Some(locations)
                })
                .flatten()
                .collect(),
        )
    }
}

impl IdentifierReferences {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
    /// Gets the identifier at the given position
    /// Returns the string and if the identifier is limited to the current file (e.g. locals)
    fn get_identifier(&self, pos: Location, uri: &Uri) -> Option<(String, bool)> {
        let doc = self.cache.get_document(uri).ok()?;
        let mut stack = doc.get_ast().ok()?.get_stack_by_position(&pos.clone());
        while let Some(top_node) = stack.peek() {
            match top_node.node_kind.as_ref() {
                NodeKind::Function(_) | NodeKind::Conditional(_) => {
                    // TODO: Same as in goto definition
                    let _ = stack.stack.pop();
                }
                _ => break,
            };
        }
        let top_node = stack.peek()?;

        Some(match top_node.node_kind.as_ref() {
            NodeKind::LiteralString(s) => (s.value.clone(), false),
            // Vars are probably always local
            NodeKind::Var(var) => (var.id.clone()?.0, true),
            NodeKind::Local(local) => (local.get_name()?, true),
            NodeKind::Function(func) => (
                func.parameters.iter().find_map(|param| {
                    if param.loc_range.in_range(&pos) {
                        Some(param.name.0.clone())
                    } else {
                        None
                    }
                })?,
                true,
            ),
            NodeKind::DesugaredObject(obj) => {
                let name = obj.get_name_at(&pos)?;
                let is_local = obj.locals.iter().any(|bind| bind.variable.0 == name);

                (name, is_local)
            }
            _ => {
                log::warn!(
                    "Unhandled identifier for {}",
                    top_node.node_kind.variant_name()
                );
                return None;
            }
        })
    }
}
