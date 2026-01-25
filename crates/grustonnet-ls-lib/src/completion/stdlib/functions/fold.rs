use std::sync::Arc;

use grustonnet_node::{stack::NodeStack, types::{function::{Apply, Arguments}, node::Node, node_kind::NodeKind, CommaSeparatedExpr}};
use language_server::cache::Cache;

use crate::{cache::JsonnetASTGenerator, completion::stdlib::{functions::{get_parameter, resolve_node}, StdArgument, StdLibCallError, StdLibFunction}};

pub struct Fold<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
    pub reverse: bool,
}

impl<'a> StdLibFunction for Fold<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![
            StdArgument {
                name: "func",
                ..Default::default()
            },
            StdArgument {
                name: "arr",
                ..Default::default()
            },
            StdArgument {
                name: "init",
                ..Default::default()
            },
        ]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let func = get_parameter(&params, 0)?;
        let array_node = get_parameter(&params, 1)?;
        let init = get_parameter(&params, 2)?;

        let resolved_func = resolve_node(self.cache, self.document_stack, func)?;
        let resolved_array = resolve_node(self.cache, self.document_stack, array_node)?;
        let resolved_init = resolve_node(self.cache, self.document_stack, init)?;

        let NodeKind::Array(arr) = resolved_array.node_kind.as_ref() else {
            return Err(StdLibCallError::InvalidArgument { reason: "arr is not an array".into() });
        };

        let fold = |init , func| {
            if self.reverse {
                arr.elements.iter().rfold(init, func)
            } else {
                arr.elements.iter().fold(init, func)
            }
        };

        let folded = fold(resolved_init, |acc, elem: &CommaSeparatedExpr | {
            let mut arg_array = vec![
                CommaSeparatedExpr {
                    expr: acc,
                    ..Default::default()
                },
            ];
            if self.reverse {
                arg_array.insert(0, elem.clone());
            } else {
                arg_array.push(elem.clone());
            }
            Node {
                node_kind: Box::new(NodeKind::Apply(
                    Apply {
                        target: resolved_func.clone(),
                        arguments: Arguments {
                            positional: arg_array,
                            ..Default::default()
                        },
                    ..Default::default()
                    }
                )),
                ..Default::default()
            }.into()
        });

        resolve_node(self.cache, self.document_stack, folded)
    }
}
