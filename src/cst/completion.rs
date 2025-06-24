use ropey::Rope;
use tree_sitter::{Node, Point};

use crate::{
    cst::{new_tree, node::JsonnetNode, node_type::NodeType, point},
    node::location::Location,
};

#[derive(Debug, Default)]
pub enum CompletionType {
    Local,
    #[default]
    Global,
    Import,
    ExtVar,
}

#[derive(Debug, Default)]
pub struct CompletionInfo<'a> {
    pub node: Option<Node<'a>>,
    pub completion_type: CompletionType,
    pub pos: Location,
}

fn get_prev_non_whitespace_position(content: &str, pos: Point) -> Point {
    let rope = Rope::from(content);
    let idx = rope.line_to_char(pos.row) + pos.column;

    let mut non_whitespace_idx = idx;
    for (i, prev_char) in rope.chars_at(idx).reversed().enumerate() {
        if !prev_char.is_whitespace() {
            non_whitespace_idx = idx - i;
            break;
        }
    }

    let line = rope.char_to_line(non_whitespace_idx);
    let char = non_whitespace_idx - rope.line_to_char(line);

    Point {
        row: line,
        column: char,
    }
}

impl<'a> CompletionInfo<'a> {
    pub fn new(content: &str, pos: Location) -> Self {
        let mut info = Self::default();
        let Some(tree) = new_tree(content) else {
            return Self::default();
        };
        let pos = get_prev_non_whitespace_position(content, pos.into());
        info.pos = pos.into();

        let root_node = tree.root_node();
        let node_at = root_node.get_node_at(pos.into()).unwrap();
        // TODO: Do we need to check the whole stack? Or is it enough to check if the next node is a dot?
        let mut current_node = node_at.clone();
        let mut nodes = vec![];
        while !current_node.is_ending_node() && current_node.prev_sibling().is_some() {
            nodes.push(current_node);
            current_node = current_node.prev_sibling().unwrap();
        }
        if nodes
            .iter()
            .any(|node| NodeType::from(*node) == NodeType::NodeDot)
        {
            info.completion_type = CompletionType::Local;
        }

        info
    }
}
