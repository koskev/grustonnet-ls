use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct LocationRange {
    pub file: Source,
    pub file_name: String,
    pub begin: Location,
    pub end: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub line: i32,
    pub column: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

impl LocationRange {
    pub fn in_range(&self, location: &Location) -> bool {
        // Same line but before range
        if self.end.line == location.line && self.begin.column > location.column {
            return false;
        }

        // Same line but before after
        if self.end.line == location.line && self.end.column < location.column {
            return false;
        }

        // In between
        return self.begin.line < location.line && self.end.line > location.line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        let range = LocationRange {
            begin: Location { line: 1, column: 3 },
            end: Location { line: 4, column: 4 },
            ..Default::default()
        };

        assert!(range.in_range(&Location {
            line: 2,
            column: 14
        }));
    }
}
