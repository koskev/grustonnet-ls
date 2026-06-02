use std::sync::Arc;

use grustonnet_node::{
    stack::NodeStack,
    types::{literals::LiteralString, node::Node, node_kind::NodeKind},
};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{
        StdArgument, StdLibCallError, StdLibFunction,
        functions::{get_parameter, resolve_node},
    },
};

pub struct Type<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
}

impl<'a> StdLibFunction for Type<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![StdArgument {
            name: "x",
            ..Default::default()
        }]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let x = get_parameter(&params, 0)?;

        let resolved_x = resolve_node(self.cache, self.document_stack, x)?;

        let name = match resolved_x.node_kind.as_ref() {
            NodeKind::LiteralString(_) => "string",
            NodeKind::Array(_) => "array",
            NodeKind::LiteralBoolean(_) => "boolean",
            NodeKind::Function(_) => "function",
            NodeKind::LiteralNull => "null",
            NodeKind::LiteralNumber(_) => "number",
            NodeKind::DesugaredObject(_) => "object",
            _ => {
                return Err(StdLibCallError::InvalidArgument {
                    reason: format!(
                        "type for {} not supported",
                        resolved_x.node_kind.variant_name()
                    ),
                });
            }
        };
        Ok(Arc::new(LiteralString::node_from_str(name)))
    }
}
