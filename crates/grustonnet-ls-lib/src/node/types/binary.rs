use serde::{Deserialize, Serialize};

use crate::node::types::{fodder::Fodder, node::Node, node_kind::NodeKind};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum BinaryOp {
    #[default]
    Mult,
    Div,
    Percent,

    Plus,
    Minus,

    ShiftL,
    ShiftR,

    Greater,
    GreaterEq,
    Less,
    LessEq,
    In,

    ManifestEqual,
    ManifestUnequal,

    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,

    And,
    Or,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Binary {
    pub left: Node,
    pub right: Node,
    pub op_fodder: Option<Fodder>,
    pub op: i32,
}

impl Binary {
    pub fn flatten(&self) -> Vec<&Node> {
        let mut nodes = vec![];
        if let NodeKind::Binary(left) = self.left.node_kind.as_ref() {
            nodes.extend(left.flatten());
        } else {
            nodes.push(&self.left);
        }
        if let NodeKind::Binary(right) = self.right.node_kind.as_ref() {
            nodes.extend(right.flatten());
        } else {
            nodes.push(&self.right);
        }

        nodes
    }
}
