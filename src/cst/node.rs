use tree_sitter::Node;

use crate::cst::node_type::NodeType;

pub enum CompletionType {
    Global,
    Local,
    Import,
    ExtVar,
}

pub struct CompletionInfo<'a> {
    node: Node<'a>,
}

pub trait JsonnetNode {
    fn is_symbol_node(&self) -> bool;
    // Get the previous node in the tree
    fn get_prev_node(&self) -> Option<Node>;
}

impl<'a> JsonnetNode for Node<'a> {
    fn is_symbol_node(&self) -> bool {
        NodeType::from(self.grammar_name()).is_symbol()
    }

    fn get_prev_node(&self) -> Option<Node> {
        match self.prev_sibling() {
            Some(sibling) => {
                let mut cursor = sibling.walk();
                cursor.goto_last_child();
                Some(cursor.node())
            }
            None => self.parent(),
        }
    }
}
