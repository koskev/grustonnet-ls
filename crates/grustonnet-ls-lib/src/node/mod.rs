use std::sync::Arc;

use anyhow::{Result, anyhow};
use grustonnet_node::{
    stack::NodeStack,
    types::{
        function::{Apply, Function},
        node::Node,
        node_kind::NodeKind,
    },
};
use language_server::cache::Cache;

use crate::{cache::JsonnetASTGenerator, completion::local::call_stack_iter::CallStackIter};

pub trait Stackhelper {
    fn get_last_unbuilt_node(&mut self, cache: &Cache<JsonnetASTGenerator>) -> Result<Arc<Node>>;

    fn build_except_last(
        &mut self,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Result<(Option<Arc<Node>>, Arc<Node>)>;
}

impl Stackhelper for NodeStack {
    fn get_last_unbuilt_node(&mut self, cache: &Cache<JsonnetASTGenerator>) -> Result<Arc<Node>> {
        let (last_node, built_node) = self.build_except_last(cache)?;
        let last_node_body = match built_node.node_kind.as_ref() {
            NodeKind::DesugaredObject(obj) => obj
                .get_field(&last_node.clone().unwrap_or_default().get_name())
                .map(|field| field.body.clone()),
            _ => Some(built_node),
        };

        last_node_body.ok_or(anyhow!("Could not get last node"))
    }

    fn build_except_last(
        &mut self,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Result<(Option<Arc<Node>>, Arc<Node>)> {
        let mut call_stack = self
            .peek()
            .ok_or(anyhow!("document stack is empty"))?
            .get_call_stack();
        let mut last_node = None;
        let built_node = match call_stack.stack.len() {
            1 => call_stack.stack.pop().expect("impossible to reach"),
            x if x > 1 => {
                // Remove the last node (=at the beginning of the vec) and resolve the rest of the stack
                last_node = Some(call_stack.stack.remove(0));
                let call_iter = CallStackIter::new_with_call_stack(cache, self, call_stack.clone())
                    .ok_or(anyhow!("could not resolve call stack"))?;
                call_iter
                    .last()
                    .ok_or(anyhow!("Call iter was empty. Stack: {}", call_stack))?
            }
            _ => {
                return Err(anyhow!("Cant find the destination of an empty stack"));
            }
        };
        Ok((last_node, built_node))
    }
}

pub trait NodeHelper {
    fn get_apply_function(
        &self,
        root_node: Arc<Node>,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Option<(Apply, Function)>;
}

impl NodeHelper for Node {
    fn get_apply_function(
        &self,
        root_node: Arc<Node>,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Option<(Apply, Function)> {
        let NodeKind::Apply(apply_node) = self.node_kind.as_ref() else {
            return None;
        };
        let mut temp_stack =
            root_node.get_stack_by_position(&apply_node.target.node_base.loc_range.end);
        // TODO: If we have a().b().c().d() we will build the node way more than needed
        let mut last_node = temp_stack.get_last_unbuilt_node(cache).ok()?;
        if let NodeKind::Var(var) = last_node.node_kind.as_ref() {
            last_node = var.resolve(&mut temp_stack)?;
        }

        // TODO: build the last node?
        let NodeKind::Function(found_function) = last_node.node_kind.as_ref() else {
            return None;
        };
        Some((apply_node.clone(), found_function.clone()))
    }
}
