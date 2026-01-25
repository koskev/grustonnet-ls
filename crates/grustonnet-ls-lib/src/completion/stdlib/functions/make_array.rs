use std::sync::Arc;

use grustonnet_node::types::{base::NodeBase, function::{Apply, Arguments}, literals::LiteralNumber, node::Node, node_kind::NodeKind, Array, CommaSeparatedExpr};

use crate::completion::stdlib::{functions::{get_parameter, get_parameter_value_parse}, StdArgument, StdLibCallError, StdLibFunction};

pub struct MakeArray;

impl StdLibFunction for MakeArray {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![
            StdArgument {
                name: "size",
                ..Default::default()
            },
            StdArgument {
                name: "function",
                ..Default::default()
            },
        ]
    }
    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let size = get_parameter_value_parse(&params, 0)?;
        let func_node = get_parameter(&params, 1)?;
        let applies = (0..size)
            .map(|i| Apply {
                target: func_node.clone(),
                arguments: Arguments {
                    positional: vec![CommaSeparatedExpr {
                        expr: Arc::new(Node {
                            node_base: NodeBase::default(),
                            node_kind: Box::new(NodeKind::LiteralNumber(LiteralNumber {
                                original_string: format!("{}", i),
                            })),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            })
            .map(|apply| CommaSeparatedExpr {
                expr: Arc::new(Node {
                    node_base: NodeBase::default(),
                    node_kind: Box::new(NodeKind::Apply(apply)),
                }),
                ..Default::default()
            })
            .collect();

        Ok(Node {
            node_kind: Box::new(NodeKind::Array(Array {
                elements: applies,
                ..Default::default()
            })),
            ..Default::default()
        }
        .into())
    }
}
