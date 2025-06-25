use ropey::Rope;

use crate::node::location::Location;

pub trait RopeHelper {
    fn replace_get_end(&mut self, old: &str, new: &str) -> Option<Location>;
    fn get_location(&self, character: usize) -> Option<Location>;
}

impl RopeHelper for Rope {
    fn get_location(&self, character: usize) -> Option<Location> {
        let line = self.char_to_line(character);
        let char = character - self.line_to_char(line);

        Some(Location {
            line: line as i32 + 1,
            column: char as i32 + 1,
        })
    }
    fn replace_get_end(&mut self, old: &str, new: &str) -> Option<Location> {
        let string_begin = self.to_string().find(&old)?;
        let string_end = string_begin + old.len();
        self.remove(string_begin..string_end);
        self.insert(string_begin, &new);
        let line = self.char_to_line(string_begin);
        let char = string_begin - self.line_to_char(line) + new.len();

        Some(Location {
            line: line as i32 + 1,
            column: char as i32 + 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use ropey::Rope;

    use crate::{node::location::Location, utils::rope::RopeHelper};

    #[test]
    fn test_rope_location() {
        let rope = Rope::from_str("01234\n6789");

        let loc = rope.get_location(4).unwrap();
        assert_eq!(loc, Location { line: 1, column: 5 });
        let loc = rope.get_location(7).unwrap();
        assert_eq!(loc, Location { line: 2, column: 2 });
    }

    #[test]
    fn test_rope_replace() {
        let mut rope = Rope::from_str("this is a test\nwith a second line");

        let new_location = rope.replace_get_end("a test", "change").unwrap();

        assert_eq!(
            new_location,
            Location {
                line: 1,
                column: 15,
            }
        );
    }
}
