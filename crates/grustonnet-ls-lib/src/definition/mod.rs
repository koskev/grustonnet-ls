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

        let mut document_stack = doc.get_ast()?.get_stack_by_position(&(pos.clone().into()));

        let (last_node, built_node) = document_stack.build_except_last(&self.cache)?;

        let index_name = last_node
            .unwrap_or(built_node.clone())
            .get_name_at_pos(&pos);

        let location: LocationRange = match built_node.node_kind.as_ref() {
            NodeKind::Var(var) => Some(
                var.resolve_bind(&document_stack)
                    .ok_or(anyhow!("unable to resolve var"))?
                    .loc_range
                    .clone(),
            ),
            NodeKind::DesugaredObject(obj) => Some(
                obj.get_field(&index_name)
                    .ok_or(anyhow!("unable to get object field {}", index_name))?
                    .loc_range
                    .clone(),
            ),
            NodeKind::Local(local) => {
                if let Some(first_bind) = local.binds.first() {
                    Some(first_bind.loc_range.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
        .ok_or(anyhow!(
            "Could not resolve location of {}",
            built_node.node_kind
        ))?;
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
