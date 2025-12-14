pub mod base;
pub mod binary;
pub mod conditional;
pub mod desugared_object;
pub mod fodder;
pub mod function;
pub mod index;
pub mod literals;
pub mod local_bind;
pub mod node;
pub mod node_kind;
pub mod var;

use std::sync::Arc;

use bincode::{Decode, Encode};
use jsonnet_location::LocationRange;
use serde::{Deserialize, Serialize};

use crate::types::{fodder::Fodder, local_bind::LocalBind, node::Node};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
pub struct CommaSeparatedExpr {
    pub expr: Arc<Node>,
    pub comma_fodder: Fodder,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct Array {
    pub elements: Vec<CommaSeparatedExpr>,
    pub close_fodder: Fodder,
    pub trailing_comma: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
pub struct Identifier(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct Local {
    pub binds: Vec<LocalBind>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct Unary {
    pub expr: Arc<Node>,
    pub op: i32,
}

impl Local {
    pub fn get_name(&self) -> Option<String> {
        Some(self.binds.first()?.variable.0.clone())
    }

    // TODO: The end might include the body even for non functions. Maybe just calculate it all
    // the time? But changing it breaks other stuff
    pub fn get_identifier_position(&self) -> Option<LocationRange> {
        // If the first bind is a function we need to fix the position
        let bind = self.binds.first()?;
        let mut range = bind.loc_range.clone();
        // TODO: this should be handles somewhere else and not at multiple locations
        if !bind.loc_range.is_valid()
            && let Some(body) = &bind.body
        {
            range = body.node_base.loc_range.clone();
            range.end.column += self.get_name().unwrap_or_default().len() as i32;
        }
        Some(range)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct Import {
    pub file: Arc<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct Error {
    expr: Arc<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EmptyNode {
    unused_node: String,
}
