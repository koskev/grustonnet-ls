use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::node::{
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
        return false;
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

    pub fn resolve_bind<'a>(&self, document_stack: &'a NodeStack) -> Option<&'a LocalBind> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &'a Vec<LocalBind>| -> Option<&'a LocalBind> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind
        };
        document_stack
            .stack
            .iter()
            .find_map(|node| match node.node_kind.as_ref() {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
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
