use std::sync::Arc;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::{fodder::Fodder, node::Node};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct Conditional {
    pub cond: Arc<Node>,
    pub branch_true: Arc<Node>,
    pub branch_false: Arc<Node>,
    pub then_fodder: Fodder,
    pub else_fodder: Fodder,
}

impl Conditional {
    pub fn resolve(&self) -> Arc<Node> {
        // TODO: Properly resolve
        self.branch_true.clone()
    }
}
