use jsonnet_location::Location;
use language_server::utils::cst::CstNodeHelper;
use ropey::Rope;
use tree_sitter::{Node, Point};

use crate::cst::{new_tree, node::JsonnetNode, node_type::NodeType};

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
        let node_at = root_node.get_node_at(pos).unwrap();
        // TODO: Do we need to check the whole stack? Or is it enough to check if the next node is a dot?
        let mut current_node = node_at;
        let mut nodes = vec![];
        while !current_node.is_ending_node() {
            nodes.push(current_node);
            if let Some(prev_sibling) = current_node.prev_sibling() {
                current_node = prev_sibling;
            } else {
                break;
            }
        }
        if current_node.is_inside_import().unwrap_or(false) {
            info.completion_type = CompletionType::Import;
        } else if let Some(dot_node) = nodes
            .iter()
            .find(|node| NodeType::from(**node) == NodeType::NodeDot)
        {
            info.completion_type = CompletionType::Local;
            log::debug!("Local completion!");
            if let Some(prev_node) = dot_node.get_prev_node() {
                log::debug!(
                    "Prev node: {} at [{},{}]",
                    prev_node.grammar_name(),
                    prev_node.start_position(),
                    prev_node.end_position()
                );
                // If the prev node is an error (which it most likely is), just skip it
                let mut prev_node = match NodeType::from(prev_node) {
                    NodeType::NodeError => prev_node.get_prev_node().unwrap_or(prev_node),
                    _ => prev_node,
                };

                // If we have a closing bracket and the prev sibling is an import: We use that node
                // as the completion pos
                if NodeType::from(prev_node) == NodeType::NodeClosingBracket {
                    log::debug!("Got closing bracket");
                    // If next sibling is import we use this as the completion node
                    if let Some(prev_sibling) = prev_node.prev_sibling() {
                        if NodeType::from(prev_sibling) == NodeType::NodeImport {
                            prev_node = prev_sibling
                        }
                    }
                }
                log::trace!(
                    "Got cst node at {:?} with type {}",
                    prev_node.start_position(),
                    prev_node.grammar_name()
                );

                info.pos = prev_node.start_position().into();
                info.pos.column += 1;
            }
        }

        info
    }
}
