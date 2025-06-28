use lsp_types::{Range, TextEdit};
use ropey::Rope;
use similar::Algorithm;

use crate::utils::rope::RopeHelper;

pub fn get_text_edits(old: &str, new: &str) -> Vec<TextEdit> {
    let operations = similar::capture_diff_slices(Algorithm::Myers, old.as_bytes(), new.as_bytes());

    let rope_old = Rope::from_str(old);
    let rope_new = Rope::from_str(new);

    let edits: Vec<TextEdit> = operations
        .iter()
        .filter_map(|op| match op {
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => Some(TextEdit {
                new_text: rope_new
                    .slice(new_index..&(new_index + new_len))
                    .as_str()?
                    .to_string(),
                range: Range {
                    start: rope_old.get_location(*old_index)?,
                    end: rope_old.get_location(*old_index + *old_len)?,
                },
            }),
            similar::DiffOp::Delete {
                old_index,
                old_len,
                new_index: _,
            } => Some(TextEdit {
                new_text: String::new(),
                range: Range {
                    start: rope_old.get_location(*old_index)?,
                    end: rope_old.get_location(*old_index + *old_len)?,
                },
            }),
            similar::DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                let pos = rope_old.get_location(*old_index)?;
                Some(TextEdit {
                    range: Range {
                        start: pos,
                        end: pos,
                    },
                    new_text: rope_new
                        .slice(new_index..&(new_index + new_len))
                        .as_str()?
                        .to_string(),
                })
            }
            _ => None,
        })
        .collect();

    edits
}
