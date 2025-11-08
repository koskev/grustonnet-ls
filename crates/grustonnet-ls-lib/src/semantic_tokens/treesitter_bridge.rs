// You should not use this feature! It is just a bad workaround for editors that lack the most essential
// features

use jsonnet_cst::new_tree;
use jsonnet_location::LocationRange;
use language_server::cache::Document;
use lazy_static::lazy_static;
use tree_sitter::{Query, QueryCursor};

use crate::{
    cache::JsonnetASTGenerator,
    semantic_tokens::{SemanticData, SemanticDataList},
};

use super::{SemanticModifier, SemanticToken};

lazy_static! {
    static ref TOKEN_MAP: Vec<(&'static str, SemanticToken, Vec<SemanticModifier>)> = vec![
        ("string", SemanticToken::String, vec![]),
        ("local", SemanticToken::Keyword, vec![]),
        ("fieldname", SemanticToken::Property, vec![]),
        ("error", SemanticToken::Keyword, vec![]),
    ];
}

pub fn get_tokens(doc: Document<JsonnetASTGenerator>) -> SemanticDataList {
    let Some(tree) = new_tree(&doc.content) else {
        return SemanticDataList::default();
    };
    let data: Vec<SemanticData> = TOKEN_MAP
        .iter()
        .flat_map(|(node, token, modifier)| {
            let query_source = format!("({}) @token", node);
            log::error!("SOURCE: {}", query_source);
            let query = Query::new(&tree.language(), &query_source).expect("BUG: Invalid query");
            let mut cursor = QueryCursor::new();
            let captures = cursor.captures(&query, tree.root_node(), doc.content.as_bytes());
            captures
                .flat_map(|(query_match, _)| {
                    log::error!("Capture1");
                    query_match.captures.iter().map(|capture| {
                        log::error!("Capture2 {:?}", capture);
                        SemanticData {
                            node_type: token.clone(),
                            node_modifier: modifier.clone(),
                            // TODO: why is length even needed? Can't we just calculate it from the
                            // location?
                            length: capture
                                .node
                                .end_position()
                                .column
                                .saturating_sub(capture.node.start_position().column)
                                as u32,
                            location: LocationRange {
                                begin: capture.node.start_position().into(),
                                end: capture.node.end_position().into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                })
                .collect::<Vec<SemanticData>>()
        })
        .collect();

    SemanticDataList { data }
}
