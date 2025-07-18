use lsp_types::Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct LocationRange {
    // We don't need this and it is stupidly large
    #[serde(skip)]
    pub file: Option<Source>,
    pub file_name: String,
    pub begin: Location,
    pub end: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub line: i32,
    pub column: i32,
}

impl Default for Location {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    pub diagnostic_file_name: String,
    pub lines: Vec<String>,
}

impl From<lsp_types::Position> for Location {
    fn from(value: lsp_types::Position) -> Self {
        Self {
            line: value.line as i32 + 1,
            column: value.character as i32 + 1,
        }
    }
}

impl Into<lsp_types::Position> for Location {
    fn into(self) -> lsp_types::Position {
        Position {
            line: std::cmp::max(0, self.line - 1) as u32,
            character: std::cmp::max(0, self.column - 1) as u32,
        }
    }
}

impl From<tree_sitter::Point> for Location {
    fn from(value: tree_sitter::Point) -> Self {
        Self {
            line: value.row as i32 + 1,
            column: value.column as i32 + 1,
        }
    }
}

impl LocationRange {
    pub fn in_range(&self, location: &Location) -> bool {
        // Same line but before range
        if self.begin.line == location.line && self.begin.column > location.column {
            return false;
        }

        // Same line but before after
        if self.end.line == location.line && self.end.column < location.column {
            return false;
        }

        // In between
        if self.begin.line != location.line || self.end.line != location.line {
            return self.begin.line <= location.line && self.end.line >= location.line;
        }

        return true;
    }

    pub fn is_valid(&self) -> bool {
        return self.begin.line != 0
            && self.begin.column != 0
            && self.end.line != 0
            && self.end.column != 0
            && self.file_name.len() != 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_multi_line() {
        let range = LocationRange {
            begin: Location { line: 1, column: 3 },
            end: Location { line: 4, column: 4 },
            ..Default::default()
        };

        assert!(range.in_range(&Location {
            line: 2,
            column: 14
        }));

        assert!(range.in_range(&Location { line: 1, column: 3 }));
        assert!(range.in_range(&Location {
            line: 1,
            column: 30
        }));
        assert!(range.in_range(&Location { line: 4, column: 4 }));

        assert!(!range.in_range(&Location { line: 1, column: 2 }));

        assert!(!range.in_range(&Location { line: 4, column: 5 }));
        assert!(!range.in_range(&Location { line: 5, column: 2 }));
    }
    #[test]
    fn test_range_single_line() {
        let range = LocationRange {
            begin: Location { line: 1, column: 1 },
            end: Location {
                line: 1,
                column: 10,
            },
            ..Default::default()
        };

        assert!(range.in_range(&Location { line: 1, column: 1 }));
        assert!(range.in_range(&Location { line: 1, column: 5 }));
        assert!(range.in_range(&Location {
            line: 1,
            column: 10,
        }));

        assert!(!range.in_range(&Location { line: 1, column: 0 }));
        assert!(!range.in_range(&Location { line: 2, column: 0 }));
        assert!(!range.in_range(&Location {
            line: 1,
            column: 11,
        }));
    }
}
