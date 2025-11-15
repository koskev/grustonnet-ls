use std::sync::Arc;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::{Identifier, fodder::Fodder, node::Node, node_kind::NodeKind};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct Index {
    pub target: Arc<Node>,
    pub index: Arc<Node>,
    pub right_bracket_fodder: Fodder,
    pub left_bracket_fodder: Fodder,
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
