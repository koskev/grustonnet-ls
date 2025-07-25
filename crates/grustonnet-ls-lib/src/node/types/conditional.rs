use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::node::types::{fodder::Fodder, node::Node};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct Conditional {
    pub cond: Arc<Node>,
    pub branch_true: Arc<Node>,
    pub branch_false: Arc<Node>,
    pub then_fodder: Option<Fodder>,
    pub else_fodder: Option<Fodder>,
}

impl Conditional {
    pub fn resolve(&self) -> Arc<Node> {
        // TODO: Properly resolve
        self.branch_true.clone()
    }
}
