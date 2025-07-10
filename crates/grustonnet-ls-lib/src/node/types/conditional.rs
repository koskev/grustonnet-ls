use serde::{Deserialize, Serialize};

use crate::node::types::{fodder::Fodder, node::Node};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Conditional {
    pub cond: Node,
    pub branch_true: Node,
    pub branch_false: Node,
    pub then_fodder: Option<Fodder>,
    pub else_fodder: Option<Fodder>,
}

impl Conditional {
    pub fn resolve(&self) -> &Node {
        // TODO: Properly resolve
        return &self.branch_true;
    }
}
