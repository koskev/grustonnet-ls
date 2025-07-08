use lsp_types::Position;
use ropey::Rope;
use tree_sitter::Node;

use crate::utils::rope::RopeHelper;

pub trait CstNodeHelper {
    fn get_name(&self, content: &str) -> Option<String>;
    fn get_prev_node(&self) -> Option<Node>;
    fn get_prev_sibling_or_parent(&self) -> Option<Node>;
}

impl<'a> CstNodeHelper for Node<'a> {
    fn get_name(&self, content: &str) -> Option<String> {
        let rope = Rope::from_str(content);

        let start_idx = rope.get_index(Position {
            line: self.start_position().row as u32,
            character: self.start_position().column as u32,
        });
        let end_idx = rope.get_index(Position {
            line: self.end_position().row as u32,
            character: self.end_position().column as u32,
        });
        rope.slice(start_idx..end_idx)
            .as_str()
            .map(|s| s.to_string())
    }

    fn get_prev_sibling_or_parent(&self) -> Option<Node<'a>> {
        match self.prev_sibling() {
            Some(sibling) => Some(sibling),
            None => self.parent(),
        }
    }

    fn get_prev_node(&self) -> Option<Node<'a>> {
        match self.prev_sibling() {
            Some(sibling) => {
                let mut cursor = sibling.walk();
                while cursor.goto_last_child() {}
                Some(cursor.node())
            }
            None => self.parent(),
        }
    }
}
