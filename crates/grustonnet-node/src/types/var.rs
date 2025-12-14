use std::sync::Arc;

use bincode::{Decode, Encode};
use jsonnet_location::LocationRange;
use serde::{Deserialize, Serialize};

use crate::{
    stack::NodeStack,
    types::{
        Identifier, function::Function, local_bind::LocalBind, node::Node, node_kind::NodeKind,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
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
        self.is_name("std") || self.is_name("$std")
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
        let source_location_range = &document_stack.stack.last()?.node_base.loc_range;
        let get_node_with_id = |binds: &'a Vec<LocalBind>| -> Option<LocationRange> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0)?;
            // If the bind is empty, we'll try the body which most likely has a valid location
            if bind.loc_range.is_valid() {
                Some(bind.loc_range.clone())
            } else {
                Some(bind.clone().body?.node_base.loc_range.clone())
            }
        };
        let handle_function = |func: &Function| -> Option<LocationRange> {
            func.parameters.iter().find_map(|param| {
                if let Some(name) = self.id.as_ref()
                    && param.name == *name
                {
                    Some(param.loc_range.clone())
                } else {
                    None
                }
            })
        };
        document_stack
            .stack
            .iter()
            .rev()
            .find_map(|node| match node.node_kind.as_ref() {
                NodeKind::DesugaredObject(obj) => {
                    // Check if the object has a field in range that has a function as a field.
                    // Then extract the parameters
                    if let Some(field_name) = obj.get_name_at(&source_location_range.begin)
                        && let Some(field) = obj.get_field(&field_name)
                        && let NodeKind::Function(func) = field.body.node_kind.as_ref()
                    {
                        handle_function(func)
                    } else {
                        get_node_with_id(&obj.locals)
                    }
                }
                NodeKind::Local(local) => get_node_with_id(&local.binds),

                NodeKind::Function(func) => handle_function(func),
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

        // To correctly resolve the vars, we need to pop all of the previous nodes (otherwise we
        // will have problems with self, dollar, and shadowed nodes. However we currently use the
        // apply node on the stack while processing a function. Therefore we are unable to find any
        // argument on the stack and can't assign them.
        // In this awful workaround we make an exception for apply nodes and just push them back to
        // the stack as they don't interfere with self etc
        // TODO: fix this mess
        let mut popped_apply_nodes = vec![];
        log::trace!(
            "Searching for {} in {}",
            self.id.clone().unwrap_or_default().0,
            document_stack
        );
        while let Some(next_node) = document_stack.stack.pop() {
            if let NodeKind::Apply(_) = *next_node.node_kind {
                popped_apply_nodes.push(next_node.clone());
            }
            if let Some(found) = match next_node.node_kind.as_ref() {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                NodeKind::Function(func) => func.parameters.iter().find_map(|p| {
                    if p.name == *id {
                        p.default_arg.clone()
                    } else {
                        None
                    }
                }),
                _ => None,
            } {
                log::trace!("Found var: {}", found.node_kind.variant_name());
                while let Some(popped) = popped_apply_nodes.pop() {
                    document_stack.push(popped);
                }
                // Push the found node back
                document_stack.push(next_node);
                return Some(found);
            }
        }
        log::trace!(
            "Unable to find var {} in stack",
            self.id.clone().unwrap_or_default().0
        );
        None
    }
}
