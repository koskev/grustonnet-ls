use std::sync::Arc;

use grustonnet_node::types::{node::Node, node_kind::NodeKind};
use language_server::cache::Cache;

use crate::{
    bridge::GenerateAST,
    cache::JsonnetASTGenerator,
    completion::stdlib::{StdArgument, StdLibCallError, StdLibFunction},
};

pub struct ExtVar<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> StdLibFunction for ExtVar<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![StdArgument {
            name: "name",
            ..Default::default()
        }]
    }
    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        //fn handle_extvar(&mut self, current_node: &Node, apply: &Apply) -> Option<Arc<Node>> {
        let conf = self.cache.ast_generator.jsonnet.get_config();
        let arg_node = params.first().ok_or(StdLibCallError::MissingArgument)?;
        if let NodeKind::LiteralString(name_node) = arg_node.node_kind.as_ref() {
            let val =
                conf.ext_code
                    .get(&name_node.value)
                    .ok_or(StdLibCallError::InvalidArgument {
                        reason: format!("Could not get extcode with name {}", name_node.value),
                    })?;
            // Get ast snippet and add to stack
            let ext_node: Arc<Node> = self
                .cache
                .ast_generator
                .jsonnet
                .get_ast_snippet_binary(&arg_node.node_base.loc_range.file_name, val)
                .map_err(|e| StdLibCallError::Wrapped { error: Box::new(e) })?
                .into();
            Ok(ext_node)
        } else {
            Err(StdLibCallError::InvalidArgument {
                reason: "Arg is not a string".into(),
            })
        }
    }
}
