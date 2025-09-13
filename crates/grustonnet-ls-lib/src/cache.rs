use std::{collections::VecDeque, sync::Arc};

use anyhow::{Result, anyhow};
use language_server::{cache::ASTGenerator, utils::rope::RopeHelper};
use ropey::Rope;

use crate::{
    bridge::{EvaluateErrorType, GenerateAST, GoJsonnet},
    node::types::node::Node,
};

#[derive(Default, Debug, Clone)]
pub struct JsonnetASTGenerator {
    pub ast: Node,
    pub jsonnet: GoJsonnet,
}

impl JsonnetASTGenerator {
    pub fn import_ast(&self, source_file: &str, filename: &str) -> Result<Node> {
        let node_data = self.jsonnet.import_ast(source_file, filename)?;
        Ok(node_data)
    }
}

impl JsonnetASTGenerator {
    fn fix_ast(
        &self,
        content: Rope,
        source_file: &str,
    ) -> Result<Arc<<JsonnetASTGenerator as language_server::cache::ASTGenerator>::Node>> {
        let mut previous_error = None;
        let mut ropes_to_test = VecDeque::new();
        ropes_to_test.push_back(content);
        // Give up after 100 tries
        for _ in 0..100 {
            if let Some(mut current_content) = ropes_to_test.pop_front() {
                log::trace!("Document content: {}", current_content);
                let json_data = self
                    .jsonnet
                    .get_ast_snippet_binary(source_file, &current_content.to_string());
                match json_data {
                    Ok(node_data) => {
                        log::debug!("Got valid ast!");
                        return Ok(node_data.into());
                    }
                    Err(e) => {
                        // If the previous error is the same as the current error, our change did
                        // nothing and we are probably in a loop
                        if let Some(previous_error) = previous_error
                            && previous_error == e
                        {
                            return Err(e.into());
                        }
                        previous_error = Some(e.clone());
                        log::warn!("Error type: {:?}", e.error_type);
                        let func_start = e.start.clone();
                        let mut add_to_prev_non_whitespace = |text: &str| {
                            let index = current_content.get_index(func_start.clone().into());
                            let non_whitespace_idx = current_content.get_prev_non_whitespace(index);
                            current_content.insert(non_whitespace_idx + 1, text);
                            ropes_to_test.push_back(current_content.clone());
                        };

                        match e.error_type {
                            EvaluateErrorType::ExpectedComma => {
                                // Insert comma before the given node after the first non whitespace
                                // character
                                add_to_prev_non_whitespace(",");
                            }
                            EvaluateErrorType::ExpectedToken => {
                                let index = current_content.get_index(e.start.into());
                                let non_whitespace_idx =
                                    current_content.get_prev_non_whitespace(index);
                                current_content.remove(non_whitespace_idx..non_whitespace_idx + 1);
                                ropes_to_test.push_back(current_content.clone());
                            }
                            EvaluateErrorType::ExpectedCommaOrSemicolon => {
                                add_to_prev_non_whitespace(";");
                                add_to_prev_non_whitespace(",");
                            }
                            _ => {
                                // TODO: Try other stuff to fix the line first and only if there is no
                                // other option for this line remove it
                                // As a last resort just try to remove the line
                                //current_content.remove_line(e.start.line.saturating_sub(1) as usize)?;
                            }
                        }
                    }
                }
            }
        }
        Err(anyhow!("Unable to fix ast after 100 tries"))
    }
}

impl ASTGenerator for JsonnetASTGenerator {
    type Node = Node;
    // BIG TODO: How to handle the modifications? AST and Editor will be out of sync
    fn update_ast(&self, source_file: &str, new_content: &str) -> Result<Arc<Self::Node>> {
        let current_content = Rope::from_str(new_content);
        self.fix_ast(current_content, source_file)
    }
}
