use std::fmt::{Display, Formatter};

use name_variant::NamedVariant;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::node::types::{
    Array, Error, Import, Local, Unary,
    binary::Binary,
    conditional::Conditional,
    desugared_object::DesugaredObject,
    function::{Apply, Function},
    index::Index,
    literals::{LiteralBoolean, LiteralNumber, LiteralString},
    var::Var,
};

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

    #[serde(alias = "Self")]
    SelfNode,
    SuperIndex,
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
        return Self::Other(json!(null));
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
}
