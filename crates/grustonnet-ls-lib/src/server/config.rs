use std::collections::HashMap;

use jsonnet_bridge::go::FormatOptions;
use lsp_types::DidChangeConfigurationParams;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Serialize, Deserialize, Clone, SmartDefault, JsonSchema)]
#[serde(default)]
pub struct CompletionConfig {
    #[default = true]
    pub enable_keywords: bool,
    #[default = true]
    pub enable_global: bool,
    #[default = true]
    pub enable_local: bool,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct DiagnosticConfig {
    #[default = true]
    pub enable_eval: bool,
    #[default = true]
    pub enable_lint: bool,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct InlayConfig {
    pub enable_debug: bool,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct JsonnetConfig {
    pub ext_code: HashMap<String, String>,
    pub ext_vars: HashMap<String, String>,
    pub jpaths: Vec<String>,

    #[default = true]
    // Searches from the root directory upwards for
    // <name>.extcode.libsonnet and
    // <name>.extvars.libsonnet
    pub find_upwards: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct Configuration {
    pub completion: CompletionConfig,
    pub diagnostics: DiagnosticConfig,
    pub jsonnet: JsonnetConfig,
    pub format: FormatOptions,
    pub inlay: InlayConfig,
}

impl TryFrom<DidChangeConfigurationParams> for Configuration {
    type Error = serde_json::Error;
    fn try_from(value: DidChangeConfigurationParams) -> Result<Self, Self::Error> {
        serde_json::from_value(value.settings)
    }
}
