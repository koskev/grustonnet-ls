use std::sync::Arc;

use grustonnet_node::{stack::NodeStack, types::{literals::LiteralBoolean, node::Node, node_kind::NodeKind}};
use language_server::cache::Cache;

use crate::{cache::JsonnetASTGenerator, completion::{local::call_stack_iter::CallStackIter, stdlib::{functions::{get_parameter, get_parameter_value, get_parameter_value_parse, object_has_ex::ObjectHasEx}, StdArgument, StdLibCallError, StdLibFunction}}};


pub struct Get<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack
}

impl<'a> StdLibFunction for Get<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![
            StdArgument {
                name: "o",
                ..Default::default()
            },
            StdArgument {
                name: "f",
                ..Default::default()
            },
            StdArgument {
                name: "default",
                default_value: Some(Node{node_kind: Box::new(NodeKind::LiteralNull), ..Default::default()}.into()),
                ..Default::default()
            },
            StdArgument {
                name: "inc_hidden",
                default_value: Some(LiteralBoolean::node_from_bool(true).into()),
                ..Default::default()
            },
        ]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let object = get_parameter(&params, 0)?;
        let name = get_parameter_value(&params, 1)?;
        let default = get_parameter(&params, 2)?;
        let inc_hidden= get_parameter_value_parse(&params, 3)?;
        // XXX: The clone might break something?
        let mut stack = self.document_stack.clone();
        stack.push(object);

        let resolved = CallStackIter::new(self.cache, &mut stack)
            .ok_or(StdLibCallError::InvalidArgument { reason: "Unable to create callsack".into() })?
            .last()
            .ok_or(StdLibCallError::InvalidArgument { reason: "Can't resolve variable".into() })?;

        let found = ObjectHasEx{
            document_stack: self.document_stack,
        }.object_has_ex(resolved.clone(), &name, inc_hidden);
        Ok(if found && let NodeKind::DesugaredObject(obj) = resolved.node_kind.as_ref() {
            obj.get_field(&name).ok_or(StdLibCallError::Unknown)?.body.clone()
        } else {
            default
        })
    }
}

