use tree_sitter::Point;

use crate::node::location::Location;

impl From<Location> for Point {
    fn from(value: Location) -> Self {
        Self {
            row: value.line as usize - 1,
            column: value.column as usize - 1,
        }
    }
}
