use std::fmt::Display;

use crate::node::Node;

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
