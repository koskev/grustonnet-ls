use std::collections::HashMap;

use lsp_types::DidChangeConfigurationParams;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Serialize, Deserialize, Clone, SmartDefault, JsonSchema)]
#[serde(default)]
pub struct SnippetConfig {
    /// Enable docsonnet snippets for new values, functions, arguments etc.
    #[default = true]
    pub docsonnet: bool,
}

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

    /// Configures the completion of various snippets
    pub snippets: SnippetConfig,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
pub enum VariableNaming {
    #[default]
    None,
    SnakeCase,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct DuplicateDetectionConfig {
    #[default = 5]
    /// The minimal length for a string to be considered
    pub min_len: usize,
    #[default = 5]
    /// The minimum number of occurrences for it to be considered a duplicate. Set to 0 to disable
    pub min_occurrences: usize,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(default)]
pub struct UnusedVariablesConfig {
    #[default = true]
    /// Enable linting for unused local variables
    pub locals: bool,
    #[default = true]
    /// Enable linting for unused function parameters
    pub function_parameters: bool,
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

    /// Enable linting for unused variables
    pub unused_variables: UnusedVariablesConfig,

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

    #[default = true]
    /// Warns if a variable gets shadowed by another variable
    pub shadow_variable: bool,

    #[default = true]
    /// Warns if the docsonnet documentation has the wrong default variable
    pub docsonnet_default: bool,

    /// Warns if a literal value is used multiple times (currently limited to a single file and
    /// literal strings)
    pub duplicate_detection: DuplicateDetectionConfig,
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

#[derive(rust2go::R2G, Serialize, Deserialize, SmartDefault, JsonSchema, Debug, Clone)]
pub struct FormatOptions {
    // Indent is the number of spaces for each level of indenation.
    #[default = 2]
    indent: i32,
    // MaxBlankLines is the max allowed number of consecutive blank lines.
    #[default = 2]
    max_blank_lines: i32,
    #[default = 1]
    string_style: i32,
    #[default = 1]
    comment_style: i32,
    // PrettyFieldNames causes fields to only be wrapped in '' when needed.
    #[default = true]
    pretty_field_names: bool,
    // PadArrays causes arrays to be written like [ this ] instead of [this].
    #[default = false]
    pad_arrays: bool,
    // PadObjects causes arrays to be written like { this } instead of {this}.
    #[default = true]
    pad_objects: bool,
    // SortImports causes imports at the top of the file to be sorted in groups
    // by filename.
    #[default = true]
    sort_imports: bool,
    // UseImplicitPlus removes plus sign where it is not required.
    #[default = true]
    use_implicit_plus: bool,

    #[default = false]
    strip_everything: bool,
    #[default = false]
    strip_comments: bool,
    #[default = false]
    strip_all_but_comments: bool,
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
