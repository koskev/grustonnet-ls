use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::node::{
    location::LocationRange,
    types::{
        CommaSeparatedExpr, Identifier, Local, base::NodeBase, fodder::Fodder,
        local_bind::LocalBind, node::Node, node_kind::NodeKind,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct NamedArgument {
    pub name_fodder: Option<Fodder>,
    pub name: Identifier,
    pub eq_fodder: Option<Fodder>,
    pub arg: Arc<Node>,
    pub comma_fodder: Option<Fodder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Apply {
    pub target: Arc<Node>,
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
    pub default_arg: Option<Arc<Node>>,
    pub loc_range: LocationRange,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Function {
    pub paren_left_fodder: Option<Fodder>,
    pub paren_right_fodder: Option<Fodder>,
    pub body: Arc<Node>,
    pub parameters: Option<Vec<Parameter>>,
    // Always false if there were no parameters.
    pub trailing_comma: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct Arguments {
    pub positional: Vec<CommaSeparatedExpr>,
    pub named: Vec<NamedArgument>,
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
    pub fn get_argument(&self, pos: usize) -> Option<Arc<Node>> {
        if let Some(arg) = self.positional.get(pos) {
            Some(arg.expr.clone())
        } else {
            Some(self.named.get(pos - self.positional.len())?.arg.clone())
        }
    }
}
