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

use serde::{Deserialize, Serialize};

use crate::node::{
    location::LocationRange,
    types::{fodder::Fodder, local_bind::LocalBind, node::Node, node_kind::NodeKind},
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct CommaSeparatedExpr {
    pub expr: Arc<Node>,
    pub comma_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Array {
    pub elements: Option<Vec<CommaSeparatedExpr>>,
    pub close_fodder: Option<Fodder>,
    pub trailing_comma: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Identifier(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Local {
    pub binds: Vec<LocalBind>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Unary {
    pub expr: Arc<Node>,
    pub op: i32,
}

impl Local {
    pub fn get_name(&self) -> Option<String> {
        Some(self.binds.first()?.variable.0.clone())
    }

    pub fn get_identifier_position(&self) -> Option<LocationRange> {
        // If the first bind is a function we need to fix the position
        let bind = self.binds.first()?;
        let mut range = bind.loc_range.clone();
        if let Some(body) = &bind.body
            && let NodeKind::Function(_func) = body.node_kind.as_ref()
        {
            range = body.node_base.loc_range.clone();
            range.end.column += self.get_name().unwrap_or_default().len() as i32;
        }
        Some(range)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Import {
    pub file: Arc<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Error {
    expr: Arc<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EmptyNode {
    unused_node: String,
}
