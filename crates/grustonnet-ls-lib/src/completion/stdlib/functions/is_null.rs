use std::sync::Arc;

use grustonnet_node::{
    stack::NodeStack,
    types::{literals::LiteralBoolean, node::Node, node_kind::NodeKind},
};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{
        StdArgument, StdLibCallError, StdLibFunction,
        functions::{get_parameter, resolve_node},
    },
};

pub struct IsNull<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
}

impl<'a> StdLibFunction for IsNull<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![StdArgument {
            name: "x",
            ..Default::default()
        }]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let x = get_parameter(&params, 0)?;

        let resolved_x = resolve_node(self.cache, self.document_stack, x)?;

        Ok(Arc::new(LiteralBoolean::node_from_bool(matches!(
            *resolved_x.node_kind,
            NodeKind::LiteralNull
        ))))
    }
}
