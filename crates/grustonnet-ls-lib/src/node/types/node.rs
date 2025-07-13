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
            NodeKind::Conditional(cond) => {
                self.index += 1;
                // TODO: Eval condition
                return match self.index {
                    1 => Some(&cond.branch_true),
                    2 => Some(&cond.branch_false),
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
