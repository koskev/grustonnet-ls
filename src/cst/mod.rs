pub mod completion;
pub mod node;
pub mod node_type;

fn new_tree(content: &str) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_jsonnet::language())
        .expect("Something is really wrong with the tresitter setup!");
    let tree = parser.parse(content, None);
    tree
}
