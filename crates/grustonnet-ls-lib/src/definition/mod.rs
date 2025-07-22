use anyhow::{Result, anyhow};
use language_server::{cache::Cache, utils::UriHelper};
use lsp_types::{Range, Uri};

use crate::{
    cache::JsonnetASTGenerator,
    node::{
        location::{Location, LocationRange},
        types::node_kind::NodeKind,
    },
};

pub struct DefinitionProvider<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
}

#[derive(Debug, Clone)]
pub struct DefinitinInfo {
    pub location: lsp_types::Location,
    pub name: String,
}

impl<'a> DefinitionProvider<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }

    pub fn definition(&self, uri: &Uri, pos: Location) -> Result<DefinitinInfo> {
        let doc = self.cache.get_document(uri)?;

        let mut document_stack = doc.get_ast()?.get_stack_by_position(&(pos.clone()));

        // Special case: If we goto the name of a local function
        // TODO: what about a function definition itself?
        if let NodeKind::Function(_func) = document_stack
            .peek()
            .ok_or(anyhow!("Empty document stack"))?
            .node_kind
            .as_ref()
        {
            // TODO: make a ast fix function and fix the local bind position
            // If we have a local with a function the bind won't have a valid location
            // Unwrap is safe
            let func_node = document_stack.stack.pop().unwrap();
            let local_node = document_stack.stack.pop().ok_or(anyhow!("no next local"))?;
            let NodeKind::Local(local) = local_node.node_kind.as_ref() else {
                return Err(anyhow!("next node is not local"));
            };
            let mut end_pos = func_node.node_base.loc_range.begin.clone();
            end_pos.column += local_node.get_name().len() as i32;
            return Ok(DefinitinInfo {
                name: local.get_name().unwrap_or_default(),
                location: lsp_types::Location {
                    uri: uri.clone(),
                    range: Range {
                        start: func_node.node_base.loc_range.begin.clone().into(),
                        end: func_node.node_base.loc_range.begin.clone().into(),
                    },
                },
            });
        }

        let (last_node, built_node) = document_stack.build_except_last(self.cache)?;

        let index_name = last_node
            .unwrap_or(built_node.clone())
            .get_name_at_pos(&pos);

        log::trace!(
            "Searching definition for {} in {}",
            index_name,
            built_node.node_kind
        );
        let location: LocationRange = match built_node.node_kind.as_ref() {
            NodeKind::Var(var) => var.resolve_location(&document_stack),
            NodeKind::DesugaredObject(obj) => Some(
                obj.get_field(&index_name)
                    .ok_or(anyhow!("unable to get object field {}", index_name))?
                    .loc_range
                    .clone(),
            ),
            NodeKind::Local(local) => local.get_identifier_position(),
            _ => None,
        }
        .ok_or(anyhow!(
            "Could not resolve location of {}",
            built_node.node_kind
        ))?;
        log::trace!("Location: {:?}", location);
        Ok(DefinitinInfo {
            name: index_name,
            location: lsp_types::Location {
                uri: Uri::from_path(&location.file_name)?,
                range: Range {
                    start: location.begin.into(),
                    end: location.end.into(),
                },
            },
        })
    }
}
