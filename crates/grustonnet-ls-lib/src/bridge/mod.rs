use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Debug,
    fs,
    path::Path,
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::Result;
use grustonnet_config::{FormatOptions, JsonnetConfig};
use grustonnet_node::types::node::Node;
use jsonnet_bridge::{
    evaluate_error::{EvaluateError, EvaluateErrorType},
    go::{ASTBridge, ASTBridgeImpl, EvaluateParams, ExtValue},
};

pub trait GenerateAST {
    fn get_ast(&self, filename: &str) -> Result<Node, EvaluateError>;
    fn get_ast_snippet(&self, source_file: &str, snippet: &str) -> Result<Node, EvaluateError>;
    fn get_ast_snippet_binary(
        &self,
        source_file: &str,
        snippet: &str,
    ) -> Result<Node, EvaluateError>;
    fn get_ast_data(&self, source_file: &str, snippet: &str) -> Result<Vec<u8>, EvaluateError>;
    fn import_ast(&self, source_file: &str, filename: &str) -> Result<Node, EvaluateError>;
    fn evaluate_ast(&self, ast_string: &str, source_file: &str) -> Result<String, EvaluateError>;
    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError>;
    fn lint_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError>;

    fn format_snippet(
        &self,
        filename: &str,
        snippet: &str,
        options: &FormatOptions,
    ) -> Result<String, EvaluateError>;
}

#[derive(Default, Debug, Clone)]
pub struct GoJsonnet {
    pub root_dir: Arc<RwLock<String>>,
    config: Arc<RwLock<JsonnetConfig>>,
    pub params: Arc<RwLock<EvaluateParams>>,
}

fn find_upwards(cwd: &str, suffix: &str) -> HashMap<String, String> {
    // TODO: generic magic
    let mut cwd_path = Path::new(cwd);
    let mut files_found = HashMap::new();
    loop {
        let Ok(dir) = fs::read_dir(cwd_path) else {
            break;
        };
        dir.into_iter()
            .filter_map(|res| res.ok())
            .filter(|entry| match entry.file_name().into_string() {
                Ok(file_name) => {
                    //log::error!("Does {} end with {}?", file_name, suffix);
                    file_name.ends_with(suffix)
                }
                Err(_) => false,
            })
            .for_each(|found| {
                let name = found
                    .file_name()
                    .into_string()
                    .unwrap()
                    .strip_suffix(suffix)
                    .unwrap()
                    .to_string();
                if let Entry::Vacant(e) = files_found.entry(name)
                    && let Ok(content) = fs::read_to_string(found.path())
                {
                    e.insert(content);
                }
            });

        match cwd_path.parent() {
            Some(parent) => cwd_path = parent,
            None => break,
        }
    }
    files_found
}

// TODO: performance nightmare
impl GoJsonnet {
    pub fn new(root_dir: &str) -> Self {
        Self {
            root_dir: Arc::new(RwLock::new(root_dir.to_string())),
            ..Default::default()
        }
    }

    pub fn set_root_dir(&self, dir: &str) {
        *self.root_dir.write().unwrap() = dir.to_string();
    }

    pub fn get_config(&self) -> JsonnetConfig {
        self.config.read().unwrap().clone()
    }

    pub fn set_config(&self, config: &JsonnetConfig) {
        let mut config_lock = self.config.write().unwrap();
        *config_lock = config.clone();

        // Find upwards
        let found_extcode = find_upwards(&self.root_dir.read().unwrap(), ".extcode.libsonnet");
        config_lock.ext_code.extend(
            found_extcode
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string())),
        );

        let mut jpaths = config.jpaths.clone();
        let root_dir = self.root_dir.read().unwrap();
        jpaths.extend(
            config
                .default_root_jpaths
                .iter()
                .map(|p| format!("{root_dir}/{p}")),
        );

        *self.params.write().unwrap() = EvaluateParams {
            ext_code: config
                .ext_code
                .iter()
                .chain(found_extcode.iter())
                .map(|(key, val)| ExtValue {
                    name: key.to_string(),
                    value: val.to_string(),
                })
                .collect(),
            ext_vars: config
                .ext_vars
                .iter()
                .map(|(key, val)| ExtValue {
                    name: key.to_string(),
                    value: val.to_string(),
                })
                .collect(),
            jpaths,
        }
    }

    pub fn get_evaluate_params(&self, filepath: &str) -> EvaluateParams {
        let mut params = self.params.read().unwrap().clone();
        // Add the current path of the file to the jpaths
        if let Ok(p) = fs::canonicalize(filepath)
            && p.is_file()
            && let Some(parent) = p.parent()
            && let Some(parent_str) = parent.to_str()
        {
            params.jpaths.insert(0, parent_str.into());
        }
        // Add environment Variables
        if let Ok(jpath_env) = std::env::var("JSONNET_PATH") {
            let parts = jpath_env
                .split(':')
                .filter_map(|s| fs::canonicalize(s).ok())
                .filter_map(|p| Some(p.to_str()?.to_string()));
            params.jpaths.extend(parts);
        }
        params
    }
}

impl GenerateAST for GoJsonnet {
    fn import_ast(&self, source_file: &str, filename: &str) -> Result<Node, EvaluateError> {
        let res = ASTBridgeImpl::import_ast(
            source_file.to_string(),
            filename.to_string(),
            self.get_evaluate_params(source_file),
        );
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        rmp_serde::from_slice(&res.ast_data).map_err(|e| EvaluateError {
            message: format!("Failed to convert data! {e}"),
            ..Default::default()
        })
    }
    fn get_ast(&self, filename: &str) -> Result<Node, EvaluateError> {
        let res = ASTBridgeImpl::get_ast(filename.to_string());
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(rmp_serde::from_slice(&res.ast_data).unwrap())
    }
    fn get_ast_data(&self, source_file: &str, snippet: &str) -> Result<Vec<u8>, EvaluateError> {
        let start = Instant::now();
        let res =
            ASTBridgeImpl::get_ast_snippet_binary(source_file.to_string(), snippet.to_string());
        let dur = start.elapsed();
        log::info!("Ast evaluation took {:?}", dur);
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(res.ast_data)
    }

    fn get_ast_snippet_binary(
        &self,
        source_file: &str,
        snippet: &str,
    ) -> Result<Node, EvaluateError> {
        let start = Instant::now();
        let res =
            ASTBridgeImpl::get_ast_snippet_binary(source_file.to_string(), snippet.to_string());
        let dur = start.elapsed();
        log::info!("Ast evaluation took {:?}", dur);
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        let start = Instant::now();
        let (node, _) = bincode::decode_from_slice(&res.ast_data, bincode::config::legacy())
            .map_err(|e| EvaluateError {
                error_type: EvaluateErrorType::Deserialize,
                message: format!("Could not decode AST. This is most likely a bug: {e}"),
                ..Default::default()
            })?;
        log::info!("Deserializing took {:?}", start.elapsed());
        Ok(node)
    }

    fn get_ast_snippet(&self, source_file: &str, snippet: &str) -> Result<Node, EvaluateError> {
        let start = Instant::now();
        let res = ASTBridgeImpl::get_ast_snippet(source_file.to_string(), snippet.to_string());
        let dur = start.elapsed();
        log::info!("Ast evaluation took {:?}", dur);
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        let start = Instant::now();
        let node = rmp_serde::from_slice(&res.ast_data).map_err(|e| EvaluateError {
            error_type: EvaluateErrorType::Deserialize,
            message: format!("Could not decode AST. This is most likely a bug: {e}"),
            ..Default::default()
        })?;
        log::info!("Deserializing took {:?}", start.elapsed());
        Ok(node)
    }

    fn evaluate_ast(&self, ast_string: &str, source_file: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::evaluate_ast(
            ast_string.to_string(),
            self.get_evaluate_params(source_file),
        );
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(String::from_utf8(res.ast_data).unwrap())
    }

    fn evaluate_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::evaluate_snippet(
            filename.to_string(),
            snippet.to_string(),
            self.get_evaluate_params(filename),
        );
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(String::from_utf8(res.ast_data).unwrap())
    }

    fn lint_snippet(&self, filename: &str, snippet: &str) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::lint_snippet(
            filename.to_string(),
            snippet.to_string(),
            self.get_evaluate_params(filename),
        );
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(String::from_utf8(res.ast_data).unwrap())
    }

    fn format_snippet(
        &self,
        filename: &str,
        snippet: &str,
        options: &FormatOptions,
    ) -> Result<String, EvaluateError> {
        let res = ASTBridgeImpl::format_snippet(
            filename.to_string(),
            snippet.to_string(),
            options.clone(),
        );
        if !res.error_data.is_empty() {
            return Err(EvaluateError::from(res.error_data));
        }
        Ok(String::from_utf8(res.ast_data).unwrap())
    }
}

#[cfg(test)]
mod test {

    use grustonnet_node::types::{base::NodeBase, fodder::Fodder, node::Node};
    use jsonnet_bridge::go::{ASTBridge, ASTBridgeImpl};
    use jsonnet_location::{Location, LocationRange};

    #[test]
    fn base_object() {
        let config = bincode::config::legacy();
        let test_objects = ASTBridgeImpl::get_test_objects();
        for test_object in test_objects {
            match test_object.name.as_str() {
                "location" => {
                    let result: (Location, usize) =
                        bincode::decode_from_slice(&test_object.data, config)
                            .expect("unable to decode location");
                    assert_eq!(result.0.line, 5);
                    assert_eq!(result.0.column, 19);
                }
                "locrange" => {
                    let (result, _): (LocationRange, usize) =
                        bincode::decode_from_slice(&test_object.data, config)
                            .unwrap_or_else(|_| panic!("Got {:?}", test_object.data));
                    assert_eq!(result.file_name, "test");
                    assert_eq!(result.begin.line, 1);
                    assert_eq!(result.begin.column, 2);
                    assert_eq!(result.end.line, 3);
                    assert_eq!(result.end.column, 4);
                }
                "base" => {
                    let _result: (NodeBase, usize) =
                        bincode::decode_from_slice(&test_object.data, config).unwrap();
                }
                "fodder" => {
                    let (result, _): (Fodder, usize) =
                        bincode::decode_from_slice(&test_object.data, config).unwrap();

                    assert_eq!(result.0.len(), 1);
                    assert_eq!(result.0[0].kind, 1);
                    assert_eq!(result.0[0].blanks, 2);
                    assert_eq!(result.0[0].indent, 3);
                    assert_eq!(result.0[0].comment.len(), 2);
                    assert_eq!(result.0[0].comment[0], "one");
                    assert_eq!(result.0[0].comment[1], "two");
                }
                "node_base" => {
                    let (result, _): (NodeBase, usize) =
                        bincode::decode_from_slice(&test_object.data, config).unwrap();
                    assert_eq!(result.ctx, "\0", "Wrong CTX");
                    assert_eq!(result.fodder.0.len(), 0);
                    assert_eq!(result.free_vars.len(), 0);
                    assert_eq!(result.loc_range.file_name, "");
                    assert_eq!(result.loc_range.begin.line, 1);
                    assert_eq!(result.loc_range.begin.column, 1);
                    assert_eq!(result.loc_range.end.line, 1);
                    assert_eq!(result.loc_range.end.column, 3);
                }
                _ => {
                    let (_result, _): (Node, usize) =
                        bincode::decode_from_slice(&test_object.data, config)
                            .unwrap_or_else(|_| panic!("Got {:?}", test_object.data));
                }
            }
        }
    }
}
