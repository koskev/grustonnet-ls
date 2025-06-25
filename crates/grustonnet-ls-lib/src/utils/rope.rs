use ropey::Rope;

use crate::node::location::Location;

pub trait RopeHelper {
    fn replace_get_end(&mut self, old: &str, new: &str) -> Option<Location>;
}

impl RopeHelper for Rope {
    fn replace_get_end(&mut self, old: &str, new: &str) -> Option<Location> {
        let string_begin = self.to_string().find(&old)?;
        let string_end = string_begin + old.len();
        self.remove(string_begin..string_end);
        self.insert(string_begin, &new);
        let line = self.char_to_line(string_begin);
        let char = string_begin - self.line_to_char(line) + new.len();

        Some(Location {
            line: line as i32 + 1,
            // Don't add +1 to be on the actual char and not behind it
            column: char as i32,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use ropey::Rope;

    use crate::{node::location::Location, utils::rope::RopeHelper};

    #[test]
    fn test_rope_replace() {
        let mut rope = Rope::from_str("this is a test\nwith a second line");

        let new_location = rope.replace_get_end("a test", "change").unwrap();

        assert_eq!(
            new_location,
            Location {
                line: 1,
                column: 14,
            }
        );
    }
}
