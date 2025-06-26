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

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct Configuration {
    pub completion: CompletionConfig,
    pub diagnostics: DiagnosticConfig,
}

impl TryFrom<DidChangeConfigurationParams> for Configuration {
    type Error = serde_json::Error;
    fn try_from(value: DidChangeConfigurationParams) -> Result<Self, Self::Error> {
        serde_json::from_value(value.settings)
    }
}
