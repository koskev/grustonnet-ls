use std::sync::Arc;

use fallible_iterator::FallibleIterator;
use grustonnet_node::{
    stack::NodeStack,
    types::{local_bind::LocalBind, node::Node, node_kind::NodeKind, var::Var},
};
use language_server::cache::Cache;

use crate::{cache::JsonnetASTGenerator, completion::local::call_stack_iter::CallStackIter};

pub trait VarHelper {
    fn resolve(
        &self,
        cache: Cache<JsonnetASTGenerator>,
        document_stack: &mut NodeStack,
    ) -> Option<Arc<Node>>;
}

impl VarHelper for Var {
    fn resolve(
        &self,
        cache: Cache<JsonnetASTGenerator>,
        document_stack: &mut NodeStack,
    ) -> Option<Arc<Node>> {
        let Some(id) = &self.id else {
            return None;
        };
        let get_node_with_id = |binds: &Vec<LocalBind>| -> Option<Arc<Node>> {
            let bind = binds.iter().find(|local| local.variable.0 == id.0);
            bind?.body.clone()
        };

        // To correctly resolve the vars, we need to pop all of the previous nodes (otherwise we
        // will have problems with self, dollar, and shadowed nodes. However we currently use the
        // apply node on the stack while processing a function. Therefore we are unable to find any
        // argument on the stack and can't assign them.
        // In this awful workaround we make an exception for apply nodes and just push them back to
        // the stack as they don't interfere with self etc
        // TODO: fix this mess
        let mut popped_apply_nodes = vec![];
        log::trace!(
            "Searching for {} in {}",
            self.id.clone().unwrap_or_default().0,
            document_stack
        );
        while let Some(next_node) = document_stack.stack.pop() {
            if let NodeKind::Apply(_) = *next_node.node_kind {
                popped_apply_nodes.push(next_node.clone());
            }
            if let Some(found) = match next_node.node_kind.as_ref() {
                NodeKind::DesugaredObject(obj) => get_node_with_id(&obj.locals),
                NodeKind::Local(local) => get_node_with_id(&local.binds),
                NodeKind::Function(func) => {
                    let found_default = func.parameters.iter().find_map(|p| {
                        if p.name == *id {
                            p.default_arg.clone()
                        } else {
                            None
                        }
                    });
                    found_default.or(
                        // If the parent is a map -> use one of the params as the default
                        // The next one is the name of the func and after that the std node
                        // The logic is kind of fucked and fragile, but works for now
                        if let Some(next_node) = document_stack.peek()
                            && let NodeKind::Apply(apply) = next_node.node_kind.as_ref()
                            && let NodeKind::Index(target) = apply.target.node_kind.as_ref()
                            && target.get_name().unwrap_or_default() == "map"
                            && let NodeKind::Var(var) = target.target.node_kind.as_ref()
                            && var.is_std()
                            && let Some(array_arg) = apply.arguments.get_argument(1)
                        // TODO: Resolve the array to support vars etc
                        {
                            let mut stack = document_stack.clone();
                            stack.push(array_arg.clone());
                            let resolved =
                                CallStackIter::new(&cache, &mut stack)?.last().ok()??;
                            if let NodeKind::Array(arr) = resolved.node_kind.as_ref()
                                && !arr.elements.is_empty()
                            {
                                Some(arr.elements[0].expr.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        },
                    )
                }
                _ => None,
            } {
                log::trace!("Found var: {}", found.node_kind.variant_name());
                while let Some(popped) = popped_apply_nodes.pop() {
                    document_stack.push(popped);
                }
                // Push the found node back
                document_stack.push(next_node);
                return Some(found);
            }
        }
        log::trace!(
            "Unable to find var {} in stack",
            self.id.clone().unwrap_or_default().0
        );
        None
    }
}
