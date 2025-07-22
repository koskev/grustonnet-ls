use std::sync::Arc;

use language_server::cache::ASTNode;
use log::error;
use serde::{Deserialize, Serialize};

use crate::node::{
    location::Location,
    stack::NodeStack,
    types::{base::NodeBase, node_kind::NodeKind},
};

impl ASTNode for Node {}
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

        // TODO: duplicate?
        search_stack.push(Arc::new(self.clone()));

        while let Some(current_node) = search_stack.stack.pop() {
            log::trace!("Handling in call stack: {}", current_node.node_kind);
            match current_node.node_kind.as_ref() {
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
        stack.push_front(Arc::new(self.clone()));

        stack
    }

    pub fn get_stack_by_position(&self, pos: &Location) -> NodeStack {
        let mut stack: NodeStack = self
            .iter()
            .filter(|child| child.node_base.loc_range.in_range(pos))
            .map(|child: &Node| child.get_stack_by_position(pos))
            .collect();
        stack.push_front(Arc::new(self.clone()));

        stack
    }

    pub fn iter<'a>(&'a self) -> NodeIter<'a> {
        NodeIter {
            root_node: self,
            index: 0,
        }
    }

    /// Additionally searches objects at the given position to find field names
    pub fn get_name_at_pos(&self, pos: &Location) -> String {
        match self.node_kind.as_ref() {
            NodeKind::DesugaredObject(obj) => obj.get_name_at(pos).unwrap_or_default(),
            _ => self.get_name(),
        }
    }
    pub fn get_name(&self) -> String {
        match self.node_kind.as_ref() {
            NodeKind::Var(var) => var.id.clone().unwrap_or_default().0,
            NodeKind::Index(idx) => idx.get_name().unwrap_or_default(),
            NodeKind::Local(local) => local.get_name().unwrap_or_default(),
            NodeKind::Apply(apply) => apply.get_name().unwrap_or_default(),

            _ => {
                log::info!("Unhandled get_name for {}", self.node_kind.variant_name());
                "".into()
            }
        }
    }
}

#[derive(Debug)]
pub struct NodeIter<'a> {
    root_node: &'a Node,
    index: usize,
}

impl<'a> IntoIterator for &'a Node {
    type Item = &'a Node;
    type IntoIter = NodeIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        NodeIter {
            root_node: self,
            index: 0,
        }
    }
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = &'a Node;
    fn next(&mut self) -> Option<Self::Item> {
        log::trace!(
            "Next item {} at {:?}",
            self.root_node.node_kind,
            self.root_node.node_base.loc_range.begin
        );
        match self.root_node.node_kind.as_ref() {
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
                    return loc.body.as_deref();
                }
                match loc.binds.get(self.index - 1) {
                    Some(bind) => {
                        self.index += 1;
                        return bind.body.as_deref();
                    }
                    None => return None,
                }
            }
            NodeKind::Function(func) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(&func.body);
                }
                if let Some(params) = &func.parameters
                    && let Some(param) = params.get(self.index - 1)
                {
                    self.index += 1;
                    return param.default_arg.as_deref();
                }
                return None;
            }
            NodeKind::DesugaredObject(obj) => {
                if let Some(field) = obj.fields.get(self.index) {
                    self.index += 1;
                    // TODO: The function does not have a valid location. Therefore we just add
                    // the function body as a child. But we probably need to fix it another way to
                    // get the parameters?
                    if let NodeKind::Function(func) = field.body.node_kind.as_ref() {
                        return Some(&func.body);
                    } else {
                        return Some(&field.body);
                    }
                    // TODO: locals, asserts
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
                let mut offset = 1;
                if let Some(arg) = apply.arguments.positional.get(self.index - offset) {
                    self.index += 1;
                    return Some(&arg.expr);
                }
                offset += apply.arguments.positional.len();
                if let Some(arg) = apply.arguments.named.get(self.index - offset) {
                    self.index += 1;
                    return Some(&arg.arg);
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
            NodeKind::Conditional(cond) => {
                self.index += 1;
                // TODO: Eval condition
                return match self.index {
                    1 => Some(&cond.branch_true),
                    2 => Some(&cond.branch_false),
                    3 => Some(&cond.cond),
                    _ => None,
                };
            }
            NodeKind::Error(err) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(&err.expr);
                }
                return None;
            }
            NodeKind::Unary(un) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(&un.expr);
                }
                return None;
            }
            NodeKind::InSuper(idx) => {
                self.index += 1;
                return match self.index {
                    1 => Some(&idx.index),
                    _ => None,
                };
            }
            NodeKind::SuperIndex(idx) => {
                self.index += 1;
                return match self.index {
                    1 => Some(&idx.index),
                    _ => None,
                };
            }
            NodeKind::LiteralString(_)
            | NodeKind::LiteralNumber(_)
            | NodeKind::LiteralBoolean(_)
            | NodeKind::LiteralNull
            | NodeKind::SelfNode
            | NodeKind::Import(_)
            | NodeKind::ImportStr(_)
            | NodeKind::ImportBin(_)
            | NodeKind::Dollar => (),
            NodeKind::Other(other) => {
                error!("Found other in children: {:?}", other)
            }
        };
        None
    }
}
