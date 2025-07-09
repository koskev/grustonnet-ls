use std::fmt::Display;

use anyhow::{Result, anyhow};
use language_server::cache::Cache;

use crate::{
    cache::JsonnetASTGenerator,
    completion::local::CallStackIter,
    node::{Node, NodeKind},
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

pub type NodeStack = NodeStackG<Node>;

impl NodeStackG<Node> {
    pub fn generate_stack_for_node(&self, node: Node) -> NodeStackG<Node> {
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

    pub fn get_last_unbuilt_node(&mut self, cache: &Cache<JsonnetASTGenerator>) -> Result<Node> {
        let (index_name, built_node) = self.build_except_last(cache)?;
        let last_node = match built_node.node_kind.as_ref() {
            NodeKind::DesugaredObject(obj) => {
                if let Some(field) = obj.get_field(&index_name) {
                    Some(field.body.clone())
                } else {
                    None
                }
            }
            _ => Some(built_node),
        };

        last_node.ok_or(anyhow!("Could not get last node"))
    }

    pub fn build_except_last(
        &mut self,
        cache: &Cache<JsonnetASTGenerator>,
    ) -> Result<(String, Node)> {
        let mut call_stack = self
            .peek()
            .ok_or(anyhow!("document stack is empty"))?
            .get_call_stack();
        let mut index_name = String::new();
        let built_node = match call_stack.stack.len() {
            x if x == 1 => call_stack.stack.pop().expect("impossible to reach"),
            x if x > 1 => {
                // Remove the last node (=at the beginning of the vec) and resolve the rest of the stack
                let last_node = call_stack.stack.remove(0);
                index_name = match last_node.node_kind.as_ref() {
                    NodeKind::Index(idx) => {
                        idx.get_name().ok_or(anyhow!("could not get index name"))?
                    }
                    NodeKind::Apply(func) => {
                        func.get_name().ok_or(anyhow!("could not get apply name"))?
                    }
                    _ => "".to_string(),
                };
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
        Ok((index_name, built_node))
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

impl FromIterator<Node> for NodeStack {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        let list: Vec<Node> = iter.into_iter().collect();
        list.into()
    }
}

impl FromIterator<NodeStack> for NodeStack {
    fn from_iter<T: IntoIterator<Item = NodeStack>>(iter: T) -> Self {
        let flat_vec: Vec<Node> = iter.into_iter().flat_map(|stack| stack.stack).collect();
        flat_vec.into()
    }
}

impl From<Vec<Node>> for NodeStack {
    fn from(value: Vec<Node>) -> Self {
        Self { stack: value }
    }
}
