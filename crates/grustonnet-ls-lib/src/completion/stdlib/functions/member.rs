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

pub struct Member<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
}

impl<'a> StdLibFunction for Member<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![
            StdArgument {
                name: "arr",
                ..Default::default()
            },
            StdArgument {
                name: "x",
                ..Default::default()
            },
        ]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let arr = get_parameter(&params, 0)?;
        let x = get_parameter(&params, 1)?;

        let resolved_arr = resolve_node(self.cache, self.document_stack, arr)?;
        let resolved_x = resolve_node(self.cache, self.document_stack, x)?;

        match resolved_arr.node_kind.as_ref() {
            NodeKind::Array(array) => {
                let found = array.elements.iter().find(|elem| {
                    let resolved_elem =
                        resolve_node(self.cache, self.document_stack, elem.expr.clone());

                    match resolved_elem {
                        Err(_e) => false,
                        Ok(resolved) => resolved.node_kind == resolved_x.node_kind,
                    }
                });

                Ok(LiteralBoolean::node_from_bool(found.is_some()).into())
            }
            NodeKind::LiteralString(string) => {
                let NodeKind::LiteralString(x_string) = resolved_x.node_kind.as_ref() else {
                    return Err(StdLibCallError::InvalidArgument {
                        reason: "arr is a string but x is not".into(),
                    });
                };
                Ok(LiteralBoolean::node_from_bool(string.value.contains(&x_string.value)).into())
            }
            _ => Err(StdLibCallError::InvalidArgument {
                reason: "arr of member is not an array or string".into(),
            }),
        }
    }
}
