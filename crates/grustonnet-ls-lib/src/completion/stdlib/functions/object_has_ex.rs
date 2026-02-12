use std::sync::Arc;

use grustonnet_node::{
    stack::NodeStack,
    types::{literals::LiteralBoolean, node::Node, node_kind::NodeKind},
};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{
        StdLibCallError, StdLibFunction,
        functions::{get_parameter, get_parameter_value, get_parameter_value_parse},
    },
    node::var::VarHelper,
};

pub struct ObjectHasEx<'a> {
    pub document_stack: &'a NodeStack,
    pub cache: &'a Cache<JsonnetASTGenerator>,
}

impl<'a> ObjectHasEx<'a> {
    fn _object_has_ex_rec(object: Arc<Node>, name: &str, include_hidden: bool) -> bool {
        let NodeKind::DesugaredObject(obj) = object.node_kind.as_ref() else {
            return false;
        };

        let Some((next_part, parts)) = name.split_once(".") else {
            return false;
        };
        let Some(found_field) = obj.fields.iter().find(|field| {
            field.name.get_name() == next_part && (field.hide == 0 || include_hidden)
        }) else {
            return false;
        };
        if parts.is_empty() {
            true
        } else {
            ObjectHasEx::_object_has_ex_rec(found_field.body.clone(), parts, include_hidden)
        }
    }
    pub fn object_has_ex(&self, object: Arc<Node>, name: &str, include_hidden: bool) -> bool {
        let object = if let NodeKind::Var(var) = object.node_kind.as_ref() {
            var.resolve(self.cache.clone(), &mut self.document_stack.clone())
        } else {
            Some(object)
        };
        let Some(object) = object else {
            return false;
        };
        let NodeKind::DesugaredObject(obj) = object.node_kind.as_ref() else {
            return false;
        };

        obj.fields
            .iter()
            .any(|field| field.name.get_name() == name && (field.hide == 1 || include_hidden))
    }
}

impl<'a> StdLibFunction for ObjectHasEx<'a> {
    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let object = get_parameter(&params, 0)?;
        let name = get_parameter_value(&params, 1)?;
        let include_hidden = get_parameter_value_parse(&params, 2)?;

        Ok(Node {
            node_kind: Box::new(NodeKind::LiteralBoolean(LiteralBoolean {
                value: self.object_has_ex(object, &name, include_hidden),
            })),
            ..Default::default()
        }
        .into())
    }
}
