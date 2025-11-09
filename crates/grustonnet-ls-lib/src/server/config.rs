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
    /// Enable completion of keywords. E.g. import, local, etc.
    pub enable_keywords: bool,
    #[default = true]
    /// Enable global completion. E.g. variables etc.
    pub enable_global: bool,
    #[default = true]
    /// Enables the local completion. E.g. foo._bar_
    pub enable_local: bool,
    #[default = true]
    /// Hides the docsonnet member in objects
    pub hide_docsonnet_members: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
pub enum VariableNaming {
    #[default]
    None,
    SnakeCase,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct DiagnosticConfig {
    #[default = true]
    /// Enable diagnostics by evaluating the jsonnet file
    pub enable_eval: bool,
    #[default = false]
    /// Enable diagnostics by using jsonnet-lint (mainly gives unused variables but may fail in
    /// some cases)
    pub enable_go_lint: bool,

    #[default = true]
    /// Enable linting checks (currently limited to unused variables)
    pub enable_lint: bool,

    /// Determines which type of variable naming diagnostics should be used
    pub variable_naming: VariableNaming,

    #[default = true]
    /// Enable linting for correcting "local myFunc = function()" to "local myFunc()"
    pub local_function: bool,

    #[default = true]
    /// Check for $ and display a hint to not use it. Yes I just hate it this much
    pub prevent_dollar: bool,

    #[default = true]
    /// Checks the default arguments of functions for recursions
    pub recursive_arguments: bool,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct InlayConfig {
    #[default = false]
    /// Enable debug inlay hints to show the ast types
    pub enable_debug: bool,
    #[default = true]
    /// Enable function parameter inlay hints (might cause some delays)
    pub enable_function_parameters: bool,

    /// Enable inlay hints at the ends of long objects etc.
    pub name_hints: InlayNameConfig,
}
#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct SemanticTokensConfig {
    #[default = true]
    /// Enable semantic tokens like extra hints for imports, self, parameters etc.
    pub semantic_tokens: bool,
    #[default = false]
    /// You should not use this option. This is only for editors that lack essential features. This
    /// maps treesitter tokens to semantic tokens
    pub treesitter_tokens: bool,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct InlayNameConfig {
    #[default = true]
    pub enabled: bool,
    #[default = 10]
    /// Sets the number of lines after which the object name is added
    pub line_threshold: i32,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct JsonnetConfig {
    pub ext_code: HashMap<String, String>,
    pub ext_vars: HashMap<String, String>,
    pub jpaths: Vec<String>,

    /// Paths relative to the root dir to add to the jpath
    #[default(vec!["lib".into(), "vendor".into(), ".".into()])]
    pub default_root_jpaths: Vec<String>,

    #[default = true]
    /// Searches from the root directory upwards for
    /// <name>.extcode.libsonnet and
    /// <name>.extvars.libsonnet
    pub find_upwards: bool,

    #[default = false]
    /// Preload all jsonnet files in all jpaths to allow for faster cross file search
    pub preload_files: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct Configuration {
    pub completion: CompletionConfig,
    pub diagnostics: DiagnosticConfig,
    pub jsonnet: JsonnetConfig,
    pub format: FormatOptions,
    pub inlay: InlayConfig,
    pub semantic_tokens: SemanticTokensConfig,
}

impl TryFrom<DidChangeConfigurationParams> for Configuration {
    type Error = serde_json::Error;
    fn try_from(value: DidChangeConfigurationParams) -> Result<Self, Self::Error> {
        serde_json::from_value(value.settings)
    }
}
