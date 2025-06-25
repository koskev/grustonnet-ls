use lsp_types::DidChangeConfigurationParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompletionConfig {
    pub enable_keywords: bool,
    pub enable_global: bool,
    pub enable_local: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub completion: CompletionConfig,
}

impl TryFrom<DidChangeConfigurationParams> for Configuration {
    type Error = serde_json::Error;
    fn try_from(value: DidChangeConfigurationParams) -> Result<Self, Self::Error> {
        serde_json::from_value(value.settings)
    }
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            enable_keywords: true,
            enable_global: true,
            enable_local: true,
        }
    }
}
