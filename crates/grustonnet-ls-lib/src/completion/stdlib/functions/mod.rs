pub mod make_array;
pub mod object_has_ex;
pub mod ext_vars;
pub mod get;
pub mod fold;

use std::{str::FromStr, sync::Arc};

use grustonnet_node::types::node::Node;

use crate::completion::stdlib::StdLibCallError;

pub fn get_parameter(params: &[Arc<Node>], num: usize) -> Result<Arc<Node>, StdLibCallError> {
    Ok(params.get(num).ok_or(StdLibCallError::MissingArgument)?.clone())
}

pub fn get_parameter_value(params: &[Arc<Node>], num: usize) -> Result<String, StdLibCallError> {
    get_parameter(params, num)?.node_kind.get_value().ok_or(StdLibCallError::InvalidArgument{reason: "Could not get value".into()})
}

pub fn get_parameter_value_parse<T>(params: &[Arc<Node>], num: usize) -> Result<T, StdLibCallError> 
where 
    T: FromStr
{
    get_parameter_value(params, num)?.parse::<T>().map_err(|_| StdLibCallError::InvalidArgument{reason: "Could not parse value".into()})
}

