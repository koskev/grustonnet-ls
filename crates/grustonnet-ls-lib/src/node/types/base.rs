use serde::{Deserialize, Serialize};

use crate::node::{location::LocationRange, types::fodder::Fodder};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", default)]
pub struct NodeBase {
    pub fodder: Option<Fodder>,
    pub ctx: Option<String>,
    pub free_vars: Option<Vec<String>>,
    pub loc_range: LocationRange,
}
