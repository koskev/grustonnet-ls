use std::sync::Arc;

use grustonnet_node::{stack::NodeStack, types::node::Node};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{
        StdArgument, StdLibCallError, StdLibFunction,
        functions::{flatten_array::FlattenArray, get_parameter, map::Map},
    },
};

pub struct FlatMap<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
}

impl<'a> StdLibFunction for FlatMap<'a> {
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

        let map_result = Map {
            cache: self.cache,
            document_stack: self.document_stack,
        }
        .call(vec![func, array])?;

        FlattenArray {
            cache: self.cache,
            document_stack: self.document_stack,
        }
        .call(vec![map_result])
    }
}
