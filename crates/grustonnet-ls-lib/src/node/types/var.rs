use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::node::{
    location::LocationRange,
    stack::NodeStack,
    types::{Identifier, local_bind::LocalBind, node::Node, node_kind::NodeKind},
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Var {
    pub id: Option<Identifier>,
}

impl Var {
    fn is_name(&self, name: &str) -> bool {
        if let Some(id) = &self.id {
            return id.0 == name;
        }
        false
    }

    // TODO: resolve before is vars
    pub fn is_std(&self) -> bool {
        self.is_name("std")
    }

    pub fn is_self(&self) -> bool {
        self.is_name("self")
    }

    pub fn is_dollar(&self) -> bool {
        self.is_name("$")
    }

    pub fn resolve_location<'a>(&self, document_stack: &'a NodeStack) -> Option<LocationRange> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &'a Vec<LocalBind>| -> Option<LocationRange> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0)?;
            // If the bind is empty, we'll try the body which most likely has a valid location
            if bind.loc_range.is_valid() {
                Some(bind.loc_range.clone())
            } else {
                Some(bind.clone().body?.node_base.loc_range.clone())
            }
        };
        document_stack
            .stack
            .iter()
            .find_map(|node| match node.node_kind.as_ref() {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                NodeKind::Function(func) => func.parameters.as_ref()?.iter().find_map(|param| {
                    if let Some(name) = self.id.as_ref()
                        && param.name == *name
                    {
                        Some(param.loc_range.clone())
                    } else {
                        None
                    }
                }),
                _ => None,
            })
    }

    pub fn resolve(&self, document_stack: &mut NodeStack) -> Option<Arc<Node>> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &Vec<LocalBind>| -> Option<Arc<Node>> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind?.body.clone()
        };
        while let Some(next_node) = document_stack.stack.pop() {
            if let Some(found) = match next_node.node_kind.as_ref() {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                NodeKind::Function(func) => func.parameters.as_ref()?.iter().find_map(|p| {
                    if p.name == *id {
                        p.default_arg.clone()
                    } else {
                        None
                    }
                }),
                _ => None,
            } {
                // Push the found node back
                document_stack.push(next_node);
                return Some(found);
            }
        }
        log::trace!("Unable to find var in stack");
        None
    }
}
