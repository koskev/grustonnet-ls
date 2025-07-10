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
    // TODO: resolve before is vars
    pub fn is_std(&self) -> bool {
        if let Some(id) = &self.id {
            return id.0 == "std";
        }
        return false;
    }

    pub fn is_self(&self) -> bool {
        if let Some(id) = &self.id {
            return id.0 == "self";
        }
        return false;
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
            .find_map(|node| match &(*node.node_kind) {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                _ => None,
            })
    }

    pub fn resolve(&self, document_stack: &NodeStack) -> Option<Node> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &Vec<LocalBind>| -> Option<Node> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind?.body.clone()
        };
        document_stack
            .stack
            .iter()
            .find_map(|node| match &(*node.node_kind) {
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
            })
    }
}
