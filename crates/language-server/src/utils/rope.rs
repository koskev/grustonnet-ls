use lsp_types::Position;
use ropey::Rope;

pub trait RopeHelper {
    fn replace_get_end(&mut self, old: &str, new: &str) -> Option<Position>;
    fn get_location(&self, character: usize) -> Option<Position>;
    // Gets the next non whitspace character before index
    fn get_prev_non_whitespace(&self, index: usize) -> usize;
    fn get_index(&self, loc: Position) -> usize;
}

impl RopeHelper for Rope {
    fn get_prev_non_whitespace(&self, index: usize) -> usize {
        let mut non_whitespace_idx = index;
        for (i, prev_char) in self.chars_at(index).reversed().enumerate() {
            if !prev_char.is_whitespace() {
                non_whitespace_idx = index - i - 1;
                break;
            }
        }
        non_whitespace_idx
    }

    fn get_index(&self, loc: Position) -> usize {
        self.line_to_char(loc.line as usize) + loc.character as usize
    }

    fn get_location(&self, character: usize) -> Option<Position> {
        let line = self.try_char_to_line(character).ok()?;
        let char = character - self.try_line_to_char(line).ok()?;

        Some(Position {
            line: line as u32,
            character: char as u32,
        })
    }
    fn replace_get_end(&mut self, old: &str, new: &str) -> Option<Position> {
        let string_begin = self.to_string().find(&old)?;
        let string_end = string_begin + old.len();
        self.remove(string_begin..string_end);
        self.insert(string_begin, &new);
        let line = self.char_to_line(string_begin);
        let char = string_begin - self.line_to_char(line) + new.len();

        Some(Position {
            line: line as u32,
            character: char as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::Position;
    use pretty_assertions::assert_eq;
    use ropey::Rope;

    use crate::utils::rope::RopeHelper;

    #[test]
    fn test_rope_whitespace() {
        let rope = Rope::from_str("0 2  5\n7 9");

        assert_eq!(rope.get_prev_non_whitespace(0), 0);
        assert_eq!(rope.get_prev_non_whitespace(1), 0);
        assert_eq!(rope.get_prev_non_whitespace(2), 0);
        assert_eq!(rope.get_prev_non_whitespace(3), 2);
        assert_eq!(rope.get_prev_non_whitespace(4), 2);
        assert_eq!(rope.get_prev_non_whitespace(6), 5);
        assert_eq!(rope.get_prev_non_whitespace(7), 5);
        assert_eq!(rope.get_prev_non_whitespace(9), 7);
    }
    #[test]
    fn test_rope_location() {
        let rope = Rope::from_str("01234\n6789");

        let loc = rope.get_location(4).unwrap();
        assert_eq!(
            loc,
            Position {
                line: 0,
                character: 4
            }
        );
        let loc = rope.get_location(7).unwrap();
        assert_eq!(
            loc,
            Position {
                line: 1,
                character: 1
            }
        );
    }

    #[test]
    fn test_rope_replace() {
        let mut rope = Rope::from_str("this is a test\nwith a second line");

        let new_location = rope.replace_get_end("a test", "change").unwrap();

        assert_eq!(
            new_location,
            Position {
                line: 0,
                character: 14,
            }
        );
    }
}
