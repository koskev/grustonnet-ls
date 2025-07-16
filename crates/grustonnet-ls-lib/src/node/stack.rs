use std::{fmt::Display, sync::Arc};

use anyhow::{Result, anyhow};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::local::CallStackIter,
    node::types::{node::Node, node_kind::NodeKind},
};

#[derive(Clone)]
pub struct NodeStackG<T>
where
    T: Clone,
{
    pub stack: Vec<T>,
}

impl<T> NodeStackG<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, node: T) {
        self.stack.push(node);
    }
    pub fn push_front(&mut self, node: T) {
        self.stack.insert(0, node);
    }

    pub fn peek(&self) -> Option<T> {
        self.stack.last().cloned()
    }
}

pub type NodeStack = NodeStackG<Arc<Node>>;

impl NodeStack {
    pub fn generate_stack_for_node(&self, node: &Node) -> NodeStack {
        self.stack
            .clone()
            .into_iter()
            .filter(|stack_node| {
                stack_node
                    .node_base
                    .loc_range
                    .in_range(&node.node_base.loc_range.begin)
            })
            .collect()
    }

    pub fn find_last_and_skip<F>(&mut self, kind: F) -> Option<Arc<Node>>
    where
        F: Fn(&NodeKind) -> bool,
    {
        let found_pos = self.stack.iter().position(|n| kind(&n.node_kind))?;

        self.stack.truncate(found_pos + 1);
        Some(self.stack[found_pos].clone())
    }

    pub fn find_next_and_skip<F>(&mut self, kind: F) -> Option<Arc<Node>>
    where
        F: Fn(&NodeKind) -> bool,
    {
        let found_pos = self.stack.iter().rposition(|n| kind(&n.node_kind))?;

        self.stack.truncate(found_pos + 1);
        Some(self.stack[found_pos].clone())
    }

    pub fn get_last_unbuilt_node(
        &mut self,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Result<Arc<Node>> {
        let (last_node, built_node) = self.build_except_last(cache)?;
        let last_node_body = match built_node.node_kind.as_ref() {
            NodeKind::DesugaredObject(obj) => {
                if let Some(field) =
                    obj.get_field(&last_node.clone().unwrap_or_default().get_name())
                {
                    Some(field.body.clone())
                } else {
                    None
                }
            }
            _ => Some(built_node),
        };

        last_node_body.ok_or(anyhow!("Could not get last node"))
    }

    pub fn build_except_last(
        &mut self,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Result<(Option<Arc<Node>>, Arc<Node>)> {
        let mut call_stack = self
            .peek()
            .ok_or(anyhow!("document stack is empty"))?
            .get_call_stack();
        let mut last_node = None;
        let built_node = match call_stack.stack.len() {
            x if x == 1 => call_stack.stack.pop().expect("impossible to reach"),
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
                return Err(anyhow!("Cant find the destination of an empty stack").into());
            }
        };
        Ok((last_node, built_node))
    }
}

impl Display for NodeStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: String = self
            .stack
            .iter()
            .map(|node| format!("{}\n", node.node_kind))
            .collect();
        write!(f, "{}", names)
    }
}

impl FromIterator<Arc<Node>> for NodeStack {
    fn from_iter<T: IntoIterator<Item = Arc<Node>>>(iter: T) -> Self {
        let list: Vec<Arc<Node>> = iter.into_iter().collect();
        list.into()
    }
}

impl FromIterator<NodeStack> for NodeStack {
    fn from_iter<T: IntoIterator<Item = NodeStack>>(iter: T) -> Self {
        let flat_vec: Vec<Arc<Node>> = iter.into_iter().flat_map(|stack| stack.stack).collect();
        flat_vec.into()
    }
}

impl From<Vec<Arc<Node>>> for NodeStack {
    fn from(value: Vec<Arc<Node>>) -> Self {
        Self { stack: value }
    }
}
