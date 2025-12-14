use std::sync::Arc;

use bincode::{Decode, Encode};
use jsonnet_location::Location;
use language_server::cache::ASTNode;
use serde::{Deserialize, Serialize};

use crate::{
    stack::NodeStack,
    types::{base::NodeBase, function::Parameter, node_kind::NodeKind, var::Var},
};

impl ASTNode for Node {}
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
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
            .map(|child| child.get_complete_stack())
            .collect();
        stack.push_front(Arc::new(self.clone()));

        stack
    }

    pub fn get_stack_by_position(&self, pos: &Location) -> NodeStack {
        let mut stack: NodeStack = self
            .iter()
            .filter_map(|child| {
                // If the child has a zero position we'll just apply the position of the parent and
                // use this as the new child
                let zero_pos = Location { line: 0, column: 0 };
                // Only clone the child if the location is actually 0
                // TODO: If we fix the position of LiteralString etc. we'll try to complete that,
                // which is obviously wrong
                let child = if child.node_base.loc_range.begin == zero_pos
                    && child.node_base.loc_range.end == zero_pos
                    && !(matches!(*child.node_kind, NodeKind::LiteralString(_))
                        || matches!(*child.node_kind, NodeKind::LiteralNumber(_))
                        || matches!(*child.node_kind, NodeKind::LiteralBoolean(_))
                        || matches!(*child.node_kind, NodeKind::LiteralNull))
                {
                    let mut child = child.as_ref().clone();
                    log::trace!(
                        "Replacing {:?} with {:?}",
                        child.node_base.loc_range,
                        self.node_base.loc_range
                    );
                    child.node_base.loc_range = self.node_base.loc_range.clone();
                    child.into()
                } else {
                    child
                };
                let in_range = child.node_base.loc_range.in_range(pos);
                log::trace!(
                    "Child {} ({:?}) in range of {:?}? {}",
                    child.node_kind.variant_name(),
                    child.node_base.loc_range,
                    pos,
                    in_range
                );

                if in_range { Some(child) } else { None }
            })
            .map(|child| child.get_stack_by_position(pos))
            .collect();

        // In some cases (e.g. desugaring an `assert` the else case gets a valid range for the
        // current pos, even if it is after the cursor. To fix that we just drop the last node if
        // it is an `error`
        // TODO: solve the actual problem?

        if stack.stack.len() > 1
            && let Some(top_node) = stack.peek()
            && matches!(top_node.node_kind.as_ref(), NodeKind::Error(_))
        {
            stack.stack.pop();
        }

        stack.push_front(Arc::new(self.clone()));

        stack
    }

    pub fn iter<'a>(&'a self) -> NodeIter<'a> {
        NodeIter {
            root_node: self,
            index: 0,
            queue: vec![],
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
            NodeKind::LiteralString(litstring) => litstring.value.clone(),

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

    queue: Vec<Arc<Node>>,
}

impl<'a> Iterator for NodeIter<'a> {
    // XXX: Item is an Arc to allow the return of function params
    type Item = Arc<Node>;
    fn next(&mut self) -> Option<Self::Item> {
        log::trace!(
            "Next item {} at {:?} ({})",
            self.root_node.node_kind,
            self.root_node.node_base.loc_range.begin,
            self.index,
        );
        if let Some(queue_node) = self.queue.pop() {
            return Some(queue_node);
        }
        let get_param_node = |param: &Parameter| -> Option<Arc<Node>> {
            Some(param.default_arg.clone().unwrap_or(Arc::new(Node {
                node_base: NodeBase {
                    loc_range: param.loc_range.clone(),
                    ..Default::default()
                },
                node_kind: Box::new(NodeKind::Var(Var {
                    id: Some(param.name.clone()),
                })),
            })))
        };
        match self.root_node.node_kind.as_ref() {
            NodeKind::Array(arr) => {
                if let Some(element) = arr.elements.get(self.index) {
                    self.index += 1;
                    return Some(element.expr.clone());
                }
            }
            NodeKind::Local(loc) => {
                if self.index == 0 {
                    self.index += 1;
                    return loc.body.clone();
                }
                match loc.binds.get(self.index - 1) {
                    Some(bind) => {
                        self.index += 1;
                        return bind.body.clone();
                    }
                    None => return None,
                }
            }
            NodeKind::Function(func) => {
                // Add the params first, as they might be used in the body
                if let Some(param) = func.parameters.get(self.index) {
                    self.index += 1;
                    return get_param_node(param);
                }
                if self.index == func.parameters.len() {
                    self.index += 1;
                    return Some(func.body.clone());
                }
                return None;
            }
            NodeKind::DesugaredObject(obj) => {
                if let Some(field) = obj.fields.get(self.index) {
                    self.index += 1;
                    // TODO: The function does not have a valid location. Therefore we just add
                    // the function body as a child. But we probably need to fix it another way to
                    // properly get the parameters
                    self.queue.push(field.name.clone());
                    if let NodeKind::Function(func) = field.body.node_kind.as_ref() {
                        let params = func.parameters.iter().filter_map(get_param_node);
                        self.queue.extend(params);
                        return Some(func.body.clone());
                    } else {
                        return Some(field.body.clone());
                    }
                }
                let offset = obj.fields.len();
                // Filter out the self nodes that are always present
                // TODO: Check if these can be used and replace the current self/$/super logic
                let filtered_locals: Vec<_> = obj
                    .locals
                    .iter()
                    .filter(|b| {
                        b.body
                            .clone()
                            .is_some_and(|n| !matches!(n.node_kind.as_ref(), NodeKind::SelfNode))
                    })
                    .collect();
                if let Some(local) = filtered_locals.get(self.index - offset) {
                    self.index += 1;
                    return local.body.clone();
                }

                let offset = offset + filtered_locals.len();
                if let Some(assert) = obj.asserts.get(self.index - offset) {
                    self.index += 1;
                    return Some(assert.clone().into());
                }
            }
            // Var has no children
            NodeKind::Var(_) => (),
            NodeKind::Index(idx) => {
                self.index += 1;
                return match self.index {
                    1 => Some(idx.target.clone()),
                    2 => Some(idx.index.clone()),
                    _ => None,
                };
            }
            NodeKind::Apply(apply) => {
                if self.index == 0 {
                    self.index += 1;
                    log::trace!("Apply target {}", apply.target.node_kind);
                    return Some(apply.target.clone());
                }
                let mut offset = 1;
                if let Some(arg) = apply.arguments.positional.get(self.index - offset) {
                    self.index += 1;
                    log::trace!("Apply arg: {}", arg.expr.node_kind);
                    return Some(arg.expr.clone());
                }
                offset += apply.arguments.positional.len();
                if let Some(arg) = apply.arguments.named.get(self.index - offset) {
                    self.index += 1;
                    return Some(arg.arg.clone());
                }
                return None;
            }
            NodeKind::Binary(binary) => {
                self.index += 1;
                return match self.index {
                    1 => Some(binary.left.clone()),
                    2 => Some(binary.right.clone()),
                    _ => None,
                };
            }
            NodeKind::Conditional(cond) => {
                self.index += 1;
                // TODO: Eval condition
                return match self.index {
                    1 => Some(cond.branch_true.clone()),
                    2 => Some(cond.branch_false.clone()),
                    3 => Some(cond.cond.clone()),
                    _ => None,
                };
            }
            NodeKind::Error(err) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(err.expr.clone());
                }
                return None;
            }
            NodeKind::Unary(un) => {
                if self.index == 0 {
                    self.index += 1;
                    return Some(un.expr.clone());
                }
                return None;
            }
            NodeKind::InSuper(idx) => {
                self.index += 1;
                return match self.index {
                    1 => Some(idx.index.clone()),
                    _ => None,
                };
            }
            NodeKind::SuperIndex(idx) => {
                self.index += 1;
                return match self.index {
                    1 => Some(idx.index.clone()),
                    _ => None,
                };
            }
            NodeKind::Import(import) => {
                self.index += 1;
                return match self.index {
                    1 => Some(import.file.clone()),
                    _ => None,
                };
            }
            NodeKind::LiteralString(_)
            | NodeKind::LiteralNumber(_)
            | NodeKind::LiteralBoolean(_)
            | NodeKind::LiteralNull
            | NodeKind::SelfNode
            | NodeKind::ImportStr(_)
            | NodeKind::ImportBin(_)
            | NodeKind::Dollar
            | NodeKind::Other => {
                log::trace!("Unhandled {}", self.root_node.node_kind.variant_name())
            }
        };
        None
    }
}
