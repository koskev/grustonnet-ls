use anyhow::{Result, anyhow};
use language_server::{cache::ASTGenerator, utils::rope::RopeHelper};
use ropey::Rope;

use crate::{
    bridge::{EvaluateErrorType, GenerateAST, GoJsonnet},
    node::Node,
};

#[derive(Default, Debug, Clone)]
pub struct JsonnetASTGenerator {
    pub ast: Node,
}

impl ASTGenerator for JsonnetASTGenerator {
    // BIG TODO: How to handle the modifications? AST and Editor will be out of sync
    fn update_ast(&mut self, new_content: &str) -> Result<()> {
        let mut current_content = Rope::from_str(new_content);
        // Give up after 100 tries
        for _ in 0..100 {
            log::trace!("Document content: {}", new_content);
            let json_data = GoJsonnet::new().get_ast_snippet(&current_content.to_string());
            match json_data {
                Ok(json_data) => {
                    log::debug!("Got valid ast!");
                    let node_data = serde_json::from_str::<Node>(&json_data)?;
                    self.ast = node_data;
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Error type: {}", e.error_type.variant_name());
                    match e.error_type {
                        EvaluateErrorType::ExpectedComma => {
                            // Insert comma before the given node after the first non whitespace
                            // character
                            let index = current_content.get_index(e.start.into());
                            let non_whitespace_idx = current_content.get_prev_non_whitespace(index);
                            current_content.insert(non_whitespace_idx + 1, ",");
                        }
                        EvaluateErrorType::ExpectedToken => {
                            let index = current_content.get_index(e.start.into());
                            let non_whitespace_idx = current_content.get_prev_non_whitespace(index);
                            current_content.remove(non_whitespace_idx..non_whitespace_idx + 1);
                        }
                        _ => return Err(e.into()),
                    }
                }
            }
        }
        Err(anyhow!("Unable to fix ast after 100 tries"))
    }
}
