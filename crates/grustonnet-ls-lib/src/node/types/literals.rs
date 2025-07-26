use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::node::types::{node::Node, node_kind::NodeKind};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
pub enum LiteralStringKind {
    #[default]
    StringSingle,
    StringDouble,
    StringBlock,
    VerbatimStringDouble,
    VerbatimStringSingle,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct LiteralString {
    pub value: String,
    pub block_indent: String,
    pub block_term_indent: String,
    pub kind: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct LiteralNumber {
    pub original_string: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct LiteralBoolean {
    pub value: bool,
}

impl LiteralString {
    pub fn node_from_str(val: &str) -> Node {
        Node {
            node_kind: Box::new(NodeKind::LiteralString(LiteralString {
                value: val.to_string(),
                ..Default::default()
            })),
            ..Default::default()
        }
    }
}
