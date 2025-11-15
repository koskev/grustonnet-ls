use std::fmt::Display;
use std::path::Path;

use anyhow::{Result, anyhow};
use grustonnet_node::types::node_kind::NodeKind;
use jsonnet_location::{Location, LocationRange};
use language_server::{cache::Cache, utils::UriHelper};
use lsp_types::{Range, Uri};

use crate::{cache::JsonnetASTGenerator, node::Stackhelper};

pub struct DefinitionProvider<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
}

#[derive(Debug, Clone)]
pub struct DefinitinInfo {
    pub location: lsp_types::Location,
    pub name: String,
}

impl Display for DefinitinInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (path: {}, range: {:?})",
            self.name,
            self.location.uri.path().as_str(),
            self.location.range
        )
    }
}

impl<'a> DefinitionProvider<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self { cache }
    }

    pub fn definition(&self, uri: &Uri, pos: Location) -> Result<DefinitinInfo> {
        let doc = self.cache.get_document(uri)?;

        let mut document_stack = doc.get_ast()?.get_stack_by_position(&(pos.clone()));

        let top_node = document_stack
            .peek()
            .ok_or(anyhow!("Empty document stack"))?;
        match top_node.node_kind.as_ref() {
            // Special case: If we goto the name of a local function
            // TODO: what about a function definition itself?
            NodeKind::Function(_func) => {
                // If we have a local with a function and want issue a definition on the local
                // identifier we have to ignore the function node as it would
                // resolve to the body content and location
                // e.g. local goto_test(arg) = {};
                //              ^
                //             <goto>
                let _ = document_stack.stack.pop();
            }
            NodeKind::LiteralString(import_str) => {
                // If we have a literal string and the parent is an import, we find the file and go
                // to it
                // TODO: check parent

                let jpaths = self
                    .cache
                    .ast_generator
                    .jsonnet
                    .get_evaluate_params(&top_node.node_base.loc_range.file_name)
                    .jpaths;
                for jpath in &jpaths {
                    let p = Path::new(jpath).join(Path::new(&import_str.value));
                    if p.exists() {
                        return Ok(DefinitinInfo {
                            name: "".into(),
                            location: lsp_types::Location {
                                uri: Uri::from_path(p).unwrap(),
                                range: Range::default(),
                            },
                        });
                    }
                }
            }
            _ => (),
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
