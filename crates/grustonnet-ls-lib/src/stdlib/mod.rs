use std::collections::HashMap;

use serde::{Deserialize, Serialize};
const STDLIB_DEFINITIONS: &'static str = include_str!(concat!(env!("OUT_DIR"), "/stdlib.json"));

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StdFunction {
    pub available_since: Option<String>,
    pub description: String,
    pub name: String,
    pub params: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StdLibGroup {
    fields: Vec<StdFunction>,
    name: String,
    id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StdLib {
    groups: Vec<StdLibGroup>,
}

#[derive(Debug, Default)]
pub struct StdFunctions {
    pub functions: HashMap<String, StdFunction>,
}

impl StdFunctions {
    pub fn generate() -> Self {
        let lib: StdLib = serde_json::from_str(STDLIB_DEFINITIONS).unwrap();

        Self {
            functions: lib
                .groups
                .iter()
                .flat_map(|group| {
                    group
                        .fields
                        .iter()
                        .flat_map(|field| [(field.name.clone(), field.clone())])
                })
                .collect(),
        }
    }
}
