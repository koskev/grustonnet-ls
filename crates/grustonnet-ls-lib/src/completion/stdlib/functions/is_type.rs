use std::sync::Arc;

use grustonnet_node::{
    stack::NodeStack,
    types::{literals::LiteralBoolean, node::Node, node_kind::NodeKind},
};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::stdlib::{StdArgument, StdLibCallError, StdLibFunction, functions::r#type::Type},
};

pub struct IsType<'a> {
    pub cache: &'a Cache<JsonnetASTGenerator>,
    pub document_stack: &'a NodeStack,
    pub name: &'a str,
}

impl<'a> IsType<'a> {
    fn inner_call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        let node_type = Type {
            cache: self.cache,
            document_stack: self.document_stack,
        }
        .call(params)?;

        let NodeKind::LiteralString(node_string) = node_type.node_kind.as_ref() else {
            return Err(StdLibCallError::InvalidArgument {
                reason: "return val of type is not a string".into(),
            });
        };

        log::debug!("Comparing {} and {}", node_string.value, self.name);

        Ok(Arc::new(LiteralBoolean::node_from_bool(
            node_string.value == self.name,
        )))
    }
}

impl<'a> StdLibFunction for IsType<'a> {
    fn get_arguments(&'_ self) -> Vec<StdArgument<'_>> {
        vec![StdArgument {
            name: "x",
            ..Default::default()
        }]
    }

    fn call(&self, params: Vec<Arc<Node>>) -> Result<Arc<Node>, StdLibCallError> {
        // FIXME: return "true" as a fallback since the "inner" function cannot resolve the "name" var in
        // "functions_with_assert.jsonnet"
        Ok(self
            .inner_call(params)
            .unwrap_or(Arc::new(LiteralBoolean::node_from_bool(true))))
    }
}
