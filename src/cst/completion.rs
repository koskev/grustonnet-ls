use tree_sitter::Node;

use crate::node::location::Location;

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
}

impl<'a> CompletionInfo<'a> {
    pub fn new(content: &str, pos: Location) -> Self {
        Self {
            ..Default::default()
        }
    }
}
