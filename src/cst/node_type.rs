pub enum NodeType {
    NodeSelf,
    NodeDollar,
    NodeDot,
    NodeColon,
    NodeOpeningBracket,
    NodeClosingBracket,
    NodeOpeningSquareBracket,
    NodeClosingSquareBracket,
    NodeSemicolon,
    NodeFieldAccess,
    NodeFunctionCall,
    NodeFunction,
    NodeID,
    NodeLocalBind,
    NodeLocal,
    NodeParenthesis,
    NodeBind,
    NodeImport,
    NodeError,
    NodeStringContent,
    NodeStringStart,
    NodeString,
    NodeArgs,
    NodeNumber,

    NodeUnknown,
}

impl From<&str> for NodeType {
    fn from(value: &str) -> Self {
        match value {
            "self" => Self::NodeSelf,
            "dollar" => Self::NodeDollar,
            "." => Self::NodeDot,
            ":" => Self::NodeColon,
            ";" => Self::NodeSemicolon,
            "(" => Self::NodeOpeningBracket,
            ")" => Self::NodeClosingBracket,
            "[" => Self::NodeOpeningSquareBracket,
            "]" => Self::NodeClosingSquareBracket,
            "fieldaccess" => Self::NodeFieldAccess,
            "functioncall" => Self::NodeFunctionCall,
            "function" => Self::NodeFunction,
            "id" => Self::NodeID,
            "local_bind" => Self::NodeLocalBind,
            "local" => Self::NodeLocal,
            "parenthesis" => Self::NodeParenthesis,
            "bind" => Self::NodeBind,
            "import" => Self::NodeImport,
            "ERROR" => Self::NodeError,
            "string_content" => Self::NodeStringContent,
            "string_start" => Self::NodeStringStart,
            "string" => Self::NodeString,
            "args" => Self::NodeArgs,
            "number" => Self::NodeNumber,

            _ => Self::NodeUnknown,
        }
    }
}

impl NodeType {
    pub fn is_symbol(&self) -> bool {
        match *self {
            Self::NodeSemicolon
            | Self::NodeDot
            | Self::NodeClosingBracket
            | Self::NodeOpeningBracket
            | Self::NodeOpeningSquareBracket
            | Self::NodeClosingSquareBracket
            | Self::NodeColon => true,
            _ => false,
        }
    }
}
