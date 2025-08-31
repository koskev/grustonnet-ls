use tree_sitter::{Node, Point};

use crate::cst::node_type::NodeType;

pub trait JsonnetNode {
    fn is_symbol_node(&self) -> bool;
    fn is_ending_node(&self) -> bool;
    // Get the previous node in the tree

    fn get_node_at(&self, point: Point) -> Option<Node<'_>>;

    fn is_inside_import(&self) -> Option<bool>;
}

impl<'a> JsonnetNode for Node<'a> {
    fn is_symbol_node(&self) -> bool {
        NodeType::from(self.grammar_name()).is_symbol()
    }

    fn is_ending_node(&self) -> bool {
        NodeType::from(self.grammar_name()).is_statement_ending()
    }

    fn get_node_at(&self, point: Point) -> Option<Node<'_>> {
        let mut start_pos = point;
        let end_pos = point;

        if start_pos.column > 0 {
            start_pos.column -= 1;
        }

        self.descendant_for_point_range(start_pos, end_pos)
    }

    fn is_inside_import(&self) -> Option<bool> {
        let node = if NodeType::from(*self) == NodeType::NodeStringStart {
            self.next_sibling()?
        } else {
            *self
        };
        let import_node = node.parent()?.parent()?;
        if NodeType::from(import_node) != NodeType::NodeImport {
            return None;
        };

        Some(true)
    }
}
