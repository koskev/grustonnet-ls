use std::sync::Arc;

use grustonnet_node::{
    stack::NodeStack,
    types::{
        Array, CommaSeparatedExpr,
        function::{Apply, Arguments},
        node::Node,
        node_kind::NodeKind,
    },
};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{
        StdArgument, StdLibCallError, StdLibFunction,
        functions::{get_parameter, resolve_node, resolve_node_mut},
    },
};

pub struct Map<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
}

impl<'a> StdLibFunction for Map<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![
            StdArgument {
                name: "func",
                ..Default::default()
            },
            StdArgument {
                name: "array",
                ..Default::default()
            },
        ]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let func = get_parameter(&params, 0)?;
        let array = get_parameter(&params, 1)?;

        let mut function_stack = self.document_stack.clone();
        let resolved_function = resolve_node_mut(self.cache, &mut function_stack, func)?;
        let resolved_array = resolve_node(self.cache, self.document_stack, array)?;

        let NodeKind::Array(array_kind) = resolved_array.node_kind.as_ref() else {
            return Err(StdLibCallError::InvalidArgument {
                reason: "Argument is not an array".into(),
            });
        };

        // For each array apply func
        let apply_array: Vec<_> = array_kind
            .elements
            .iter()
            .filter_map(|elem| {
                let resolved_elem =
                    resolve_node(self.cache, &function_stack, elem.expr.clone()).ok()?;
                log::error!("#### RESOLVED: {}", resolved_elem);
                Some(Apply {
                    target: resolved_function.clone(),
                    arguments: Arguments {
                        positional: vec![CommaSeparatedExpr {
                            expr: resolved_elem,
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                })
            })
            .map(|apply| CommaSeparatedExpr {
                expr: Arc::new(Node {
                    node_kind: Box::new(NodeKind::Apply(apply)),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .collect();

        Ok(Node {
            node_kind: Box::new(NodeKind::Array(Array {
                elements: apply_array,
                ..Default::default()
            })),
            ..Default::default()
        }
        .into())
    }
}
