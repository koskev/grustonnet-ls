use std::fmt::{Debug, Display, Formatter, write};

use language_server::cache::ASTNode;
use log::*;
use name_variant::NamedVariant;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::node::{
    location::{Location, LocationRange},
    stack::NodeStack,
};

pub mod location;
pub mod stack;

impl ASTNode for Node {}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", default)]
pub struct NodeBase {
    pub fodder: Option<Fodder>,
    pub ctx: Option<String>,
    pub free_vars: Option<Vec<String>>,
    pub loc_range: LocationRange,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", default)]
pub struct Node {
    pub node_base: NodeBase,

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
                    call_stack.push(current_node.clone());

                    search_stack.push(idx.target.clone());
                }
                NodeKind::Var(_var) => {
                    call_stack.push(current_node);
                    // TODO: handle array
                }
                NodeKind::Apply(apply) => {
                    log::debug!("Apply target {}", apply.target.node_kind.variant_name());
                    // If apply target is an index we need to add it to the search stack. E.g. for
                    // myVar.myFunc()
                    if matches!(*apply.target.node_kind, NodeKind::Index(_)) {
                        search_stack.push(apply.target.clone());
                    }
                    call_stack.push(current_node);
                }
                _ => {
                    log::debug!(
                        "Unhandled in build call stack: {}",
                        current_node.node_kind.variant_name()
                    );
                    call_stack.push(current_node);
                }
            }
        }

        call_stack
    }

    pub fn get_complete_stack(&self) -> NodeStack {
        let mut stack: NodeStack = self
            .iter()
            .map(|child: &Node| child.get_complete_stack())
            .collect();
        stack.push_front(self.clone());

        stack
    }

    pub fn get_stack_by_position(&self, pos: &Location) -> NodeStack {
        let mut stack: NodeStack = self
            .iter()
            .filter(|child| {
                let in_range = child.node_base.loc_range.in_range(pos);
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
            NodeKind::Index(idx) => {
                self.index += 1;
                return match self.index {
                    1 => Some(&idx.target),
                    2 => Some(&idx.index),
                    _ => None,
                };
            }
            NodeKind::Apply(apply) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(&apply.target);
                }
                return None;
            }
            NodeKind::Binary(binary) => {
                self.index += 1;
                return match self.index {
                    1 => Some(&binary.left),
                    2 => Some(&binary.right),
                    _ => None,
                };
            }
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Fodder(pub Vec<FodderElement>);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct FodderElement {
    pub comment: Vec<String>,
    pub kind: i32,
    pub blanks: i32,
    pub indent: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum FodderKind {
    #[default]
    FodderLineEnd,
    FodderInterstitial,
    FodderParagraph,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct CommaSeparatedExpr {
    pub expr: Node,
    pub comma_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct LocalBind {
    pub var_fodder: Option<Fodder>,
    pub body: Option<Node>,
    pub eq_fodder: Option<Fodder>,
    pub variable: Identifier,
    pub close_fodder: Option<Fodder>,
    pub fun: Option<Function>,
    pub loc_range: LocationRange,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum LiteralStringKind {
    #[default]
    StringSingle,
    StringDouble,
    StringBlock,
    VerbatimStringDouble,
    VerbatimStringSingle,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct LiteralString {
    pub value: String,
    pub block_indent: String,
    pub block_term_indent: String,
    pub kind: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Array {
    pub elements: Option<Vec<CommaSeparatedExpr>>,
    pub close_fodder: Option<Fodder>,
    pub trailing_comma: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Arguments {
    pub positional: Vec<CommaSeparatedExpr>,
    pub named: Vec<NamedArgument>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Identifier(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct NamedArgument {
    pub name_fodder: Option<Fodder>,
    pub name: Identifier,
    pub eq_fodder: Option<Fodder>,
    pub arg: Node,
    pub comma_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Parameter {
    pub name_fodder: Option<Fodder>,
    pub name: Identifier,
    pub comma_fodder: Option<Fodder>,
    pub eq_fodder: Option<Fodder>,
    pub default_arg: Option<Node>,
    pub loc_range: LocationRange,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Function {
    pub paren_left_fodder: Option<Fodder>,
    pub paren_right_fodder: Option<Fodder>,
    pub body: Node,
    pub parameters: Option<Vec<Parameter>>,
    // Always false if there were no parameters.
    pub trailing_comma: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct DesugaredObjectField {
    pub name: Node,
    pub body: Node,
    pub loc_range: LocationRange,
    pub hide: i32,
    pub plus_super: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct DesugaredObject {
    pub asserts: Vec<Node>,
    pub fields: Vec<DesugaredObjectField>,
    pub locals: Vec<LocalBind>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Index {
    pub target: Node,
    pub index: Node,
    pub right_bracket_fodder: Option<Fodder>,
    pub left_bracket_fodder: Option<Fodder>,
    pub id: Option<Identifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Var {
    pub id: Option<Identifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Local {
    pub binds: Vec<LocalBind>,
    pub body: Option<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Binary {
    pub left: Node,
    pub right: Node,
    pub op_fodder: Option<Fodder>,
    pub op: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Import {
    pub file: Node,
}

#[derive(Debug, Serialize, Deserialize, Clone, NamedVariant, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub enum NodeKind {
    Binary(Binary),
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
    Import(Import),

    #[serde(alias = "Self")]
    SelfNode,

    // Leftover nodes. Most likely something is broken
    Other(serde_json::Value),
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.variant_name())?;
        match self {
            Self::Local(local) => {
                write!(f, "Binds:")?;
            }
            Self::LiteralString(s) => {
                write!(f, "{}", s.value)?;
            }
            Self::Apply(apply) => {
                write!(f, "({:?}) -> {}", apply.arguments, apply.target.node_kind)?;
            }
            Self::Index(idx) => {
                write!(f, "{} -> {}", idx.index.node_kind, idx.target.node_kind)?;
            }
            Self::Var(var) => {
                if let Some(id) = &var.id {
                    write!(f, "{}", id.0)?;
                }
            }
            Self::Function(func) => {
                write!(f, "{}", func.body.node_kind)?;
            }
            Self::DesugaredObject(obj) => {
                write!(f, "{}", obj)?;
            }
            Self::Binary(binary) => {
                write!(
                    f,
                    "left {} right {}",
                    binary.left.node_kind, binary.right.node_kind
                )?;
            }
            _ => (),
        };
        Ok(())
    }
}

impl Default for NodeKind {
    fn default() -> Self {
        return Self::Other(json!(null));
    }
}

impl Var {
    // TODO: resolve before is vars
    pub fn is_std(&self) -> bool {
        if let Some(id) = &self.id {
            return id.0 == "std";
        }
        return false;
    }

    pub fn is_self(&self) -> bool {
        if let Some(id) = &self.id {
            return id.0 == "self";
        }
        return false;
    }

    pub fn resolve_bind<'a>(&self, document_stack: &'a NodeStack) -> Option<&'a LocalBind> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &'a Vec<LocalBind>| -> Option<&'a LocalBind> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind
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

    pub fn resolve(&self, document_stack: &NodeStack) -> Option<Node> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &Vec<LocalBind>| -> Option<Node> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind?.body.clone()
        };
        document_stack
            .stack
            .iter()
            .find_map(|node| match &(*node.node_kind) {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                NodeKind::Function(func) => func.parameters.as_ref()?.iter().find_map(|p| {
                    if p.name == *id {
                        p.default_arg.clone()
                    } else {
                        None
                    }
                }),
                _ => None,
            })
    }
}

impl Index {
    pub fn get_name(&self) -> Option<String> {
        match &(*self.index.node_kind) {
            NodeKind::LiteralString(name) => Some(name.value.clone()),
            _ => None,
        }
    }
}

impl DesugaredObjectField {
    pub fn get_name(&self) -> Option<String> {
        match self.name.node_kind.as_ref() {
            NodeKind::LiteralString(name) => Some(name.value.clone()),
            _ => None,
        }
    }
}

impl Display for DesugaredObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let names: String = self
            .fields
            .iter()
            .map(|f| format!("field {}", f.name.node_kind))
            .collect();
        write!(f, "{}", names)
    }
}

impl DesugaredObject {
    pub fn merge(&self, other: DesugaredObject) -> DesugaredObject {
        let mut new_object = self.clone();
        log::debug!("Merging {} and {}", self, other);
        new_object.asserts.extend(other.asserts);
        new_object.fields.extend(other.fields);
        new_object.locals.extend(other.locals);

        new_object
    }

    pub fn get_field(&self, name: &str) -> Option<&DesugaredObjectField> {
        self.fields.iter().find(|field| {
            if let Some(field_name) = field.get_name() {
                field_name == name
            } else {
                false
            }
        })
    }
}

impl Apply {
    pub fn get_name(&self) -> Option<String> {
        if let NodeKind::Index(idx) = self.target.node_kind.as_ref() {
            idx.get_name()
        } else {
            None
        }
    }
}

impl Function {
    pub fn get_bind_for_arguments(&self, arguments: &Arguments) -> Option<Vec<Node>> {
        let mut bindings = vec![];
        for (i, expr) in arguments.positional.iter().enumerate() {
            let var = self.parameters.clone()?.get(i)?.name.clone();
            log::debug!("Pushed arg {}", var.0);
            bindings.push(Node {
                node_kind: Box::new(NodeKind::Local(Local {
                    binds: vec![LocalBind {
                        variable: var,
                        body: Some(expr.expr.clone()),
                        ..Default::default()
                    }],
                    ..Default::default()
                })),
                node_base: NodeBase {
                    ctx: Some("manually pushed".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        let named_nodes = arguments.named.iter().map(|arg| {
            log::debug!("Pushed arg {}", arg.name.0);
            Node {
                node_kind: Box::new(NodeKind::Local(Local {
                    binds: vec![LocalBind {
                        variable: arg.name.clone(),
                        body: Some(arg.arg.clone()),
                        ..Default::default()
                    }],
                    ..Default::default()
                })),
                ..Default::default()
            }
        });
        bindings.extend(named_nodes);

        Some(bindings)
    }
}

impl Arguments {
    pub fn get_argument(&self, pos: usize) -> Option<Node> {
        if let Some(arg) = self.positional.get(pos) {
            Some(arg.expr.clone())
        } else {
            Some(self.named.get(pos - self.positional.len())?.arg.clone())
        }
    }
}

impl LiteralString {
    pub fn node_from_str(val: &str) -> Node {
        Node {
            node_kind: Box::new(NodeKind::LiteralString(LiteralString {
                value: val.to_string(),
                ..Default::default()
            })),
            ..Default::default()
        }
    }
}

impl Binary {
    pub fn flatten(&self) -> Vec<&Node> {
        let mut nodes = vec![];
        if let NodeKind::Binary(left) = self.left.node_kind.as_ref() {
            nodes.extend(left.flatten());
        } else {
            nodes.push(&self.left);
        }
        if let NodeKind::Binary(right) = self.right.node_kind.as_ref() {
            nodes.extend(right.flatten());
        } else {
            nodes.push(&self.right);
        }

        nodes
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
    use std::{fs, path::Path};

    use crate::node::Node;

    #[test]
    fn test_ast_parsing() {
        let dir = fs::read_dir("src/node/test_json/").unwrap();

        for test_file in dir {
            let file_path = test_file.unwrap().path();
            let content = fs::read_to_string(&file_path).expect("File not found!");

            let _node: Node = serde_json::from_str(&content).expect(&format!(
                "{} should parse to an ast",
                file_path.to_str().unwrap()
            ));
        }
    }
}
