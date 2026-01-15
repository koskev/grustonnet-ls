use jsonnet_cst::new_tree;
use jsonnet_location::Location;
use language_server::cache::Cache;
use lsp_types::Range;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

use crate::{cache::JsonnetASTGenerator, references::ReferenceType};

pub struct ImportReferences {
    pub cache: Cache<JsonnetASTGenerator>,
}

impl ImportReferences {
    pub fn new(cache: Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }
}

impl ReferenceType for ImportReferences {
    fn is_valid(&self, loc: lsp_types::Location) -> bool {
        loc.range == Range::default()
    }
    fn generate_potential_locations(
        &self,
        target_info: &crate::definition::DefinitionInfo,
        files: &[lsp_types::Uri],
    ) -> Option<Vec<lsp_types::Location>> {
        // Target is not the start of a file
        if target_info.location.range != Range::default() {
            return None;
        }
        let query_source = "(import (string (string_content) @import))";

        // Get all import statements
        let mut potential_locations: Vec<_> = files
            .iter()
            .filter_map(|uri| {
                let content = self
                    .cache
                    .get_document_with_option(uri, false)
                    .ok()?
                    .content;
                // TODO: Cache the tree for a performance gain
                let tree = new_tree(&content)?;
                let query = Query::new(&tree.language(), query_source)
                    .unwrap_or_else(|_| panic!("BUG: Invalid query: {}", query_source));
                let mut cursor = QueryCursor::new();
                let captures = cursor.captures(&query, tree.root_node(), content.as_bytes());
                let mut locations: Vec<lsp_types::Location> = vec![];
                captures.for_each(|query_match| {
                    query_match.0.captures.iter().for_each(|capture| {
                        let start: Location = capture.node.start_position().into();
                        let end: Location = capture.node.end_position().into();
                        locations.push(lsp_types::Location {
                            uri: uri.clone(),
                            range: Range {
                                start: start.into(),
                                end: end.into(),
                            },
                        });
                    })
                });
                Some(locations)
            })
            .flatten()
            .collect();

        // Add the target as a location as it is not included in the treesitter query
        potential_locations.push(target_info.location.clone());

        Some(potential_locations)
    }
}
