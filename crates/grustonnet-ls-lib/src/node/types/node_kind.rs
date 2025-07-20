use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use lsp_types::CompletionItemKind;
use name_variant::NamedVariant;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::node::types::{
    Array, Error, Identifier, Import, Local, Unary,
    binary::Binary,
    conditional::Conditional,
    desugared_object::DesugaredObject,
    fodder::Fodder,
    function::{Apply, Function},
    index::Index,
    literals::{LiteralBoolean, LiteralNumber, LiteralString},
    node::Node,
    var::Var,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct InSuper {
    pub index: Arc<Node>,
    pub in_fodder: Option<Fodder>,
    pub super_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct SuperIndex {
    #[serde(rename = "IDFodder")]
    pub id_fodder: Option<Fodder>,
    pub index: Arc<Node>,
    pub dot_fodder: Option<Fodder>,
    pub id: Option<Identifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, NamedVariant, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub enum NodeKind {
    Binary(Binary),
    Array(Array),
    LiteralNumber(LiteralNumber),
    LiteralString(LiteralString),
    LiteralBoolean(LiteralBoolean),
    LiteralNull,
    Local(Local),
    Function(Function),
    Apply(Apply),
    DesugaredObject(DesugaredObject),
    Index(Index),
    Var(Var),
    Import(Import),
    ImportStr(Import),
    ImportBin(Import),
    Conditional(Conditional),
    Error(Error),
    Unary(Unary),
    InSuper(InSuper),

    #[serde(alias = "Self")]
    SelfNode,
    SuperIndex(SuperIndex),
    Dollar,

    // Leftover nodes. Most likely something is broken
    Other(serde_json::Value),
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.variant_name())?;
        match self {
            Self::Local(local) => {
                write!(f, "Binds: {:?}", local.get_name())?;
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
        Self::Other(json!(null))
    }
}

impl NodeKind {
    pub fn get_value(&self) -> Option<String> {
        match self {
            Self::LiteralString(litstr) => Some(litstr.value.clone()),
            Self::LiteralNumber(litnum) => Some(litnum.original_string.clone()),
            Self::LiteralBoolean(litbool) => Some(litbool.value.to_string()),
            Self::DesugaredObject(obj) => Some(obj.to_string()),
            _ => None,
        }
    }

    // Not using into/from to make calling this method easier
    pub fn get_lsp_kind(&self) -> CompletionItemKind {
        match self {
            NodeKind::Apply(_) | NodeKind::Function(_) => CompletionItemKind::FUNCTION,
            NodeKind::Import(_) => CompletionItemKind::MODULE,
            _ => CompletionItemKind::VARIABLE,
        }
    }

    /// Gets a readable name to insert into the completion item
    pub fn get_node_kind_name(&self) -> &str {
        match self {
            NodeKind::LiteralNumber(_) => "number",
            NodeKind::LiteralString(_) => "string",
            NodeKind::LiteralBoolean(_) => "boolean",
            NodeKind::LiteralNull => "null",
            NodeKind::DesugaredObject(_) => "object",
            NodeKind::ImportStr(_) | NodeKind::Import(_) | NodeKind::ImportBin(_) => "import",
            NodeKind::Var(_) => "variable",
            NodeKind::SuperIndex(_) | NodeKind::InSuper(_) => "super",
            NodeKind::Index(_) => "index",
            NodeKind::Binary(_) => "binary",
            NodeKind::Array(_) => "array",
            NodeKind::Apply(_) | NodeKind::Function(_) => "function",
            NodeKind::Conditional(_) => "conditional",
            NodeKind::Unary(_) => "unary",
            NodeKind::Error(_) => "error",
            NodeKind::SelfNode => "self",
            NodeKind::Dollar => "dollar",
            NodeKind::Local(_) => "local",
            NodeKind::Other(_) => "invalid",
        }
    }
}
