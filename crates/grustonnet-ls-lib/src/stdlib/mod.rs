use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct StdLibField {
    available_since: String,
    description: String,
    name: String,
    params: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StdLibGroup {
    fields: Vec<StdLibField>,
    name: String,
    id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StdLib {
    groups: Vec<StdLibGroup>,
}
