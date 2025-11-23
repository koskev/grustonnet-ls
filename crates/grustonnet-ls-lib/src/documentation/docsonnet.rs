use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema, EnumString)]
#[strum(ascii_case_insensitive)]
enum Type {
    String,
    Number,
    Boolean,
    Object,
    Array,
    #[default]
    Any,
    Function,
}

type Fields = HashMap<String, Field>;

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
struct Field {
    function: Function,
    object: Object,
    value: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
struct Object {
    help: String,
    fields: Fields,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
struct Function {
    name: String,
    help: String,
    args: Vec<Argument>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
struct Argument {
    r#type: Type,
    name: String,
    default: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
struct Value {
    help: String,

    r#type: Type,
    default: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
struct Package {
    name: String,
    import: String,
    help: String,
}

