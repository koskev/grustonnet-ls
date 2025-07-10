use serde::{Deserialize, Serialize};

use crate::node::types::{Identifier, fodder::Fodder, node::Node, node_kind::NodeKind};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Index {
    pub target: Node,
    pub index: Node,
    pub right_bracket_fodder: Option<Fodder>,
    pub left_bracket_fodder: Option<Fodder>,
    pub id: Option<Identifier>,
}

impl Index {
    pub fn get_name(&self) -> Option<String> {
        match &(*self.index.node_kind) {
            NodeKind::LiteralString(name) => Some(name.value.clone()),
            _ => None,
        }
    }
}
