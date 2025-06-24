use std::fmt::{Debug, Formatter};

use log::*;
use name_variant::NamedVariant;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    cst::node_type::NodeType,
    node::{
        location::{Location, LocationRange},
        stack::NodeStack,
    },
};

pub mod location;
pub mod stack;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Node {
    pub fodder: Option<Fodder>,
    pub ctx: Option<String>,
    pub free_vars: Option<Vec<String>>,
    pub loc_range: LocationRange,

    #[serde(flatten)]
    pub node_kind: Box<NodeKind>,
}

impl Node {
    pub fn get_call_stack(&self) -> NodeStack {
        let mut call_stack = NodeStack::new();
        let mut search_stack = NodeStack::new();

        search_stack.push(self.clone());

        while let Some(current_node) = search_stack.stack.pop() {
            match &(*current_node.node_kind) {
                NodeKind::Index(idx) => {
                    search_stack.push(idx.target.clone());
                    call_stack.push(current_node);
                }
                NodeKind::Var(_var) => {
                    call_stack.push(current_node);
                }
                _ => call_stack.push(current_node),
            }
        }

        call_stack
    }

    pub fn get_stack_by_position(&self, pos: &Location) -> NodeStack {
        let mut stack: NodeStack = self
            .iter()
            .filter(|child| {
                let in_range = child.loc_range.in_range(pos);
                in_range
            })
            .map(|child: &Node| child.get_stack_by_position(pos))
            .collect();
        stack.push_front(self.clone());

        stack
    }
    pub fn iter<'a>(&'a self) -> NodeIter<'a> {
        NodeIter {
            root_node: self,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct NodeIter<'a> {
    root_node: &'a Node,
    index: usize,
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = &'a Node;
    fn next(&mut self) -> Option<Self::Item> {
        match &(*self.root_node.node_kind) {
            NodeKind::Array(arr) => {
                if let Some(elements) = &arr.elements {
                    if let Some(element) = elements.get(self.index) {
                        self.index += 1;
                        return Some(&element.expr);
                    }
                }
            }
            NodeKind::Local(loc) => {
                if self.index == 0 {
                    self.index += 1;
                    return loc.body.as_ref();
                }
                match loc.binds.get(self.index - 1) {
                    Some(bind) => {
                        self.index += 1;
                        return bind.body.as_ref();
                    }
                    None => return None,
                }
            }
            NodeKind::Function(func) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(&func.body);
                }
                return None;
            }
            NodeKind::DesugaredObject(obj) => {
                if let Some(field) = obj.fields.get(self.index) {
                    self.index += 1;
                    return Some(&field.body);
                }
            }
            // Var has no children
            NodeKind::Var(_) => (),
            _ => {
                error!(
                    "Unhandled type {} while searching for children",
                    self.root_node.node_kind.variant_name()
                )
            }
        };
        return None;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Fodder(pub Vec<FodderElement>);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct FodderElement {
    pub comment: Vec<String>,
    pub kind: i32,
    pub blanks: i32,
    pub indent: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub enum FodderKind {
    #[default]
    FodderLineEnd,
    FodderInterstitial,
    FodderParagraph,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CommaSeparatedExpr {
    pub expr: Node,
    pub comma_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub enum BinaryOp {
    #[default]
    Mult,
    Div,
    Percent,

    Plus,
    Minus,

    ShiftL,
    ShiftR,

    Greater,
    GreaterEq,
    Less,
    LessEq,
    In,

    ManifestEqual,
    ManifestUnequal,

    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,

    And,
    Or,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct LocalBind {
    pub var_fodder: Option<Fodder>,
    pub body: Option<Node>,
    pub eq_fodder: Option<Fodder>,
    pub variable: Identifier,
    pub close_fodder: Option<Fodder>,
    pub fun: Option<Function>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub enum LiteralStringKind {
    #[default]
    StringSingle,
    StringDouble,
    StringBlock,
    VerbatimStringDouble,
    VerbatimStringSingle,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct LiteralString {
    pub value: String,
    pub block_indent: String,
    pub block_term_indent: String,
    pub kind: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Array {
    pub elements: Option<Vec<CommaSeparatedExpr>>,
    pub close_fodder: Option<Fodder>,
    pub trailing_comma: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Arguments {
    pub positional: Vec<CommaSeparatedExpr>,
    pub named: Vec<NamedArgument>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Identifier(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct NamedArgument {
    pub name_fodder: Option<Fodder>,
    pub name: Identifier,
    pub eq_fodder: Option<Fodder>,
    pub arg: Node,
    pub comma_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Apply {
    pub target: Node,
    pub fodder_left: Option<Fodder>,
    pub arguments: Arguments,
    pub fodder_right: Option<Fodder>,
    pub tail_strict_fodder: Option<Fodder>,
    // Always false if there were no arguments.
    pub trailing_comma: bool,
    pub tail_strict: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Parameter {
    pub name_fodder: Option<Fodder>,
    pub name: Identifier,
    pub comma_fodder: Option<Fodder>,
    pub eq_fodder: Option<Fodder>,
    pub default_arg: Option<Node>,
    pub loc_range: LocationRange,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Function {
    pub paren_left_fodder: Option<Fodder>,
    pub paren_right_fodder: Option<Fodder>,
    pub body: Node,
    pub parameters: Option<Vec<Parameter>>,
    // Always false if there were no parameters.
    pub trailing_comma: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct DesugaredObjectField {
    pub name: Node,
    pub body: Node,
    pub loc_range: LocationRange,
    pub hide: i32,
    pub plus_super: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct DesugaredObject {
    pub asserts: Vec<Node>,
    pub fields: Vec<DesugaredObjectField>,
    pub locals: Vec<LocalBind>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Index {
    pub target: Node,
    pub index: Node,
    pub right_bracket_fodder: Option<Fodder>,
    pub left_bracket_fodder: Option<Fodder>,
    pub id: Option<Identifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Var {
    pub id: Option<Identifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Local {
    pub binds: Vec<LocalBind>,
    pub body: Option<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone, NamedVariant)]
#[serde(rename_all = "PascalCase", untagged)]
pub enum NodeKind {
    #[serde(rename_all = "PascalCase")]
    Binary {
        left: Node,
        right: Node,
        op_fodder: Option<Fodder>,
        op: i32,
    },
    Array(Array),
    #[serde(rename_all = "PascalCase")]
    LiteralNumber {
        original_string: String,
    },
    LiteralString(LiteralString),
    Local(Local),
    Function(Function),
    Apply(Apply),
    DesugaredObject(DesugaredObject),
    Index(Index),
    Var(Var),
    Other(serde_json::Value),
}

impl Default for NodeKind {
    fn default() -> Self {
        return Self::Other(json!(null));
    }
}

impl Var {
    pub fn resolve(&self, document_stack: &NodeStack) -> Option<Node> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &Vec<LocalBind>| -> Option<Node> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind.unwrap().body.clone()
        };
        document_stack
            .stack
            .iter()
            .find_map(|node| match &(*node.node_kind) {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                _ => None,
            })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EmptyNode {
    unused_node: String,
}

pub struct TypedDebugWrapper<'a, T: ?Sized>(&'a T);

impl<T: Debug> Debug for TypedDebugWrapper<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", core::any::type_name::<T>())
    }
}

pub trait TypedDebug: Debug {
    fn typed_debug(&self) -> TypedDebugWrapper<'_, Self> {
        TypedDebugWrapper(self)
    }
}

impl<T: ?Sized + Debug> TypedDebug for T {}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_node(data: &str) -> Node {
        let node = serde_json::from_str::<Node>(data);
        node.unwrap()
    }

    fn get_array_node() -> Node {
        Node {
            node_kind: Box::new(NodeKind::Array(Array {
                trailing_comma: false,
                close_fodder: None,
                elements: None,
            })),
            ..Default::default()
        }
    }

    #[test]
    fn test_local_from_str() {
        let data = include_str!("./test_local.json");
        let node_data = serde_json::from_str::<Node>(data).unwrap();
        match *node_data.node_kind {
            NodeKind::Local(_) => (),
            _ => assert!(
                false,
                "Node is of kind {}",
                node_data.node_kind.variant_name()
            ),
        }
    }

    #[test]
    fn test_binary_from_str() {
        let data = include_str!("./test_binary.json");
        let node_data = serde_json::from_str::<Node>(data).unwrap();
        match *node_data.node_kind {
            NodeKind::Binary {
                left,
                right,
                op_fodder,
                op,
            } => (),
            _ => assert!(false),
        }
    }
    #[test]
    fn test_binary() {
        let node = Node {
            node_kind: Box::new(NodeKind::Binary {
                left: get_array_node(),
                right: get_array_node(),
                op_fodder: None,
                op: 0,
            }),
            ..Default::default()
        };

        let json_str = serde_json::to_string(&node).unwrap();
        let json_val: Node = serde_json::from_str(&json_str).unwrap();
        match *json_val.node_kind {
            NodeKind::Binary {
                left,
                right,
                op_fodder,
                op,
            } => (),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_array() {
        let node = Node {
            node_kind: Box::new(NodeKind::Array(Array {
                trailing_comma: false,
                close_fodder: None,
                elements: None,
            })),
            ..Default::default()
        };

        let json_str = serde_json::to_string(&node).unwrap();
        println!("Node str: {}", json_str);
        let json_val: Node = serde_json::from_str(&json_str).unwrap();
        match *json_val.node_kind {
            NodeKind::Array(_) => (),
            _ => assert!(false),
        }
    }
}
