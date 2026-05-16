pub mod ext_vars;
pub mod flat_map;
pub mod flatten_array;
pub mod fold;
pub mod get;
pub mod make_array;
pub mod map;
pub mod member;
pub mod object_has_ex;

use std::{str::FromStr, sync::Arc};

use fallible_iterator::FallibleIterator;
use grustonnet_node::{stack::NodeStack, types::node::Node};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::{local::call_stack_iter::CallStackIter, stdlib::StdLibCallError},
};

pub fn get_parameter(params: &[Arc<Node>], num: usize) -> Result<Arc<Node>, StdLibCallError> {
    Ok(params
        .get(num)
        .ok_or(StdLibCallError::MissingArgument)?
        .clone())
}

pub fn get_parameter_value(params: &[Arc<Node>], num: usize) -> Result<String, StdLibCallError> {
    get_parameter(params, num)?
        .node_kind
        .get_value()
        .ok_or(StdLibCallError::InvalidArgument {
            reason: "Could not get value".into(),
        })
}

pub fn get_parameter_value_parse<T>(params: &[Arc<Node>], num: usize) -> Result<T, StdLibCallError>
where
    T: FromStr,
{
    get_parameter_value(params, num)?
        .parse::<T>()
        .map_err(|_| StdLibCallError::InvalidArgument {
            reason: "Could not parse value".into(),
        })
}

pub fn resolve_node(
    cache: &Cache<JsonnetASTGenerator>,
    stack: &NodeStack,
    node: Arc<Node>,
) -> Result<Arc<Node>, StdLibCallError> {
    let mut stack = stack.clone();
    resolve_node_mut(cache, &mut stack, node)
}

pub fn resolve_node_mut(
    cache: &Cache<JsonnetASTGenerator>,
    stack: &mut NodeStack,
    node: Arc<Node>,
) -> Result<Arc<Node>, StdLibCallError> {
    stack.push(node);
    CallStackIter::new(cache, stack)
        .ok_or(StdLibCallError::InvalidArgument {
            reason: "Unable to create callsack".into(),
        })?
        .last()
        .map_err(|e| StdLibCallError::Wrapped { error: Box::new(e) })?
        .ok_or(StdLibCallError::InvalidArgument {
            reason: "Can't resolve variable".into(),
        })
}
