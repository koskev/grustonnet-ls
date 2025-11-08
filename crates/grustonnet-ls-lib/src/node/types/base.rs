use bincode::{Decode, Encode};
use jsonnet_location::LocationRange;
use serde::{Deserialize, Serialize};

use crate::node::types::fodder::Fodder;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", default)]
pub struct NodeBase {
    pub fodder: Fodder,
    pub ctx: String,
    pub free_vars: Vec<String>,
    pub loc_range: LocationRange,
}
