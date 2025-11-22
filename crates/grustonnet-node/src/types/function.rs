use std::sync::Arc;

use bincode::{Decode, Encode};
use jsonnet_location::LocationRange;
use serde::{Deserialize, Serialize};

use crate::types::{
    CommaSeparatedExpr, Identifier, Local, base::NodeBase, fodder::Fodder, local_bind::LocalBind,
    node::Node, node_kind::NodeKind,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct NamedArgument {
    pub name_fodder: Fodder,
    pub name: Identifier,
    pub eq_fodder: Fodder,
    pub arg: Arc<Node>,
    pub comma_fodder: Fodder,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct Apply {
    pub target: Arc<Node>,
    pub fodder_left: Fodder,
    pub arguments: Arguments,
    pub fodder_right: Fodder,
    pub tail_strict_fodder: Fodder,
    // Always false if there were no arguments.
    pub trailing_comma: bool,
    pub tail_strict: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct Parameter {
    pub name_fodder: Fodder,
    pub name: Identifier,
    pub comma_fodder: Fodder,
    pub eq_fodder: Fodder,
    pub default_arg: Option<Arc<Node>>,
    pub loc_range: LocationRange,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T", default)]
pub struct Function {
    pub paren_left_fodder: Fodder,
    pub paren_right_fodder: Fodder,
    pub body: Arc<Node>,
    pub parameters: Vec<Parameter>,
    // Always false if there were no parameters.
    pub trailing_comma: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct Arguments {
    pub positional: Vec<CommaSeparatedExpr>,
    pub named: Vec<NamedArgument>,
}

impl Apply {
    pub fn get_name(&self) -> Option<String> {
        match self.target.node_kind.as_ref() {
            NodeKind::Index(idx) => idx.get_name(),
            NodeKind::Var(var) => Some(var.id.clone()?.0),
            _ => {
                log::warn!(
                    "Could not get name of apply with target {}",
                    self.target.node_kind.variant_name()
                );
                None
            }
        }
    }
}

impl Function {
    pub fn get_bind_for_arguments(&self, arguments: &Arguments) -> Option<Vec<Node>> {
        let mut bindings = vec![];
        for (i, expr) in arguments.positional.iter().enumerate() {
            let var = self.parameters.clone().get(i)?.name.clone();
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
                    ctx: "manually pushed".to_string(),
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
    pub fn get_argument(&self, pos: usize) -> Option<Arc<Node>> {
        if let Some(arg) = self.positional.get(pos) {
            Some(arg.expr.clone())
        } else {
            Some(self.named.get(pos - self.positional.len())?.arg.clone())
        }
    }
}
