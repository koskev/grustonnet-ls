use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::node::{
    location::LocationRange,
    types::{Identifier, fodder::Fodder, function::Function, node::Node},
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct LocalBind {
    pub var_fodder: Option<Fodder>,
    pub body: Option<Arc<Node>>,
    pub eq_fodder: Option<Fodder>,
    pub variable: Identifier,
    pub close_fodder: Option<Fodder>,
    pub fun: Option<Function>,
    pub loc_range: LocationRange,
}
