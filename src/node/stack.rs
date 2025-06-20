use crate::node::Node;

#[derive(Debug, Clone)]
pub struct NodeStack {
    pub stack: Vec<Node>,
}

impl NodeStack {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, node: Node) {
        self.stack.push(node);
    }
    pub fn push_front(&mut self, node: Node) {
        self.stack.insert(0, node);
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
