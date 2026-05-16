use std::sync::Arc;

use grustonnet_node::{
    stack::NodeStack,
    types::{Array, node::Node, node_kind::NodeKind},
};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{
        StdArgument, StdLibCallError, StdLibFunction,
        functions::{get_parameter, resolve_node},
    },
};

pub struct FlattenArray<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
}

impl<'a> StdLibFunction for FlattenArray<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![StdArgument {
            name: "arr",
            ..Default::default()
        }]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let array_node = get_parameter(&params, 0)?;

        let resolved_array = resolve_node(self.cache, self.document_stack, array_node)?;

        let NodeKind::Array(arr) = resolved_array.node_kind.as_ref() else {
            return Err(StdLibCallError::InvalidArgument {
                reason: "arr is not an array".into(),
            });
        };

        let flattened_expr = arr
            .elements
            .iter()
            .flat_map(|elem| {
                let resolved = resolve_node(self.cache, self.document_stack, elem.expr.clone());
                if let Ok(resolved) = resolved
                    && let NodeKind::Array(inner_arr) = resolved.node_kind.as_ref()
                {
                    inner_arr.elements.clone()
                } else {
                    vec![]
                }
            })
            .collect();

        Ok(Node {
            node_kind: Box::new(NodeKind::Array(Array {
                elements: flattened_expr,
                ..Default::default()
            })),
            ..Default::default()
        }
        .into())
    }
}
