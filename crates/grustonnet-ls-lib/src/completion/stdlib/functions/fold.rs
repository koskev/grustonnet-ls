use std::sync::Arc;

use grustonnet_node::types::{function::{Apply, Arguments}, node::Node, node_kind::NodeKind, CommaSeparatedExpr};

use crate::completion::stdlib::{functions::get_parameter, StdArgument, StdLibCallError, StdLibFunction};

pub struct Foldl;

impl StdLibFunction for Foldl {
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

        let NodeKind::Array(arr) = array_node.node_kind.as_ref() else {
            return Err(StdLibCallError::InvalidArgument { reason: "arr is not an array".into() });
        };


        Ok(arr.elements.iter().fold(init, |acc, elem|{
            Node {
                node_kind: Box::new(NodeKind::Apply(
                    Apply {
                        target: func.clone(),
                        arguments: Arguments {
                            positional: vec![
                                CommaSeparatedExpr {
                                expr: acc,
                                ..Default::default()
                            },
                            elem.clone()],
                            ..Default::default()
                        },
                    ..Default::default()
                    }
                )),
                ..Default::default()
            }.into()
        }))
    }
}

pub struct Foldr;

impl StdLibFunction for Foldr {
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

        let NodeKind::Array(arr) = array_node.node_kind.as_ref() else {
            return Err(StdLibCallError::InvalidArgument { reason: "arr is not an array".into() });
        };


        Ok(arr.elements.iter().rfold(init, |acc, elem|{
            Node {
                node_kind: Box::new(NodeKind::Apply(
                    Apply {
                        target: func.clone(),
                        arguments: Arguments {
                            positional: vec![
                                elem.clone(),
                                CommaSeparatedExpr {
                                expr: acc,
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                    ..Default::default()
                    }
                )),
                ..Default::default()
            }.into()
        }))
    }
}
