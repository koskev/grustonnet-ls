use std::{
    fs::{self},
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use grustonnet_node::{
    stack::NodeStack,
    types::{desugared_object::DesugaredObject, node::Node, node_kind::NodeKind},
};
use jsonnet_location::{Location, LocationRange};
use language_server::{
    cache::Cache,
    completion::{Completion, CompletionContext, CompletionResult},
};
use lsp_types::{CompletionItem, CompletionList, Uri};
use sha2::{Digest, Sha256};
use tree_sitter::{Query, QueryCursor, QueryMatch, StreamingIterator};
use utils::{RwLockPanic, cst::CstNodeHelper, uri::UriHelper};

use crate::{
    bridge::GenerateAST, cache::JsonnetASTGenerator, completion::stdlib::functions::resolve_node,
};

const MARKER_PREFIX: &str = "grustonnetParentObject";

pub struct GlobalObjectRef<'a> {
    cache: &'a Cache<JsonnetASTGenerator>,
    resolvers: Arc<RwLock<Vec<Box<dyn ParentObjectResolver>>>>,
}

pub struct ParentObjectInfo {
    /// The content of the parent object. Possibly external to the current file
    parent_object: Arc<Node>,
    /// The child object we want to complete in
    child_object: DesugaredObject,
}

pub trait ParentObjectResolver: Send + Sync {
    fn get_parent_object(&self, stack: &NodeStack) -> Option<ParentObjectInfo>;
}

pub struct MarkerParentResolver {}

impl ParentObjectResolver for MarkerParentResolver {
    fn get_parent_object(&self, stack: &NodeStack) -> Option<ParentObjectInfo> {
        stack.stack.iter().rev().find_map(|node| {
            if let NodeKind::DesugaredObject(obj) = node.node_kind.as_ref() {
                let parent_field = obj.get_field(&format!("_{}", MARKER_PREFIX))?;
                // We could save the path here, but that would make it more complex to support more
                // strategies to get the object
                Some(ParentObjectInfo {
                    parent_object: parent_field.body.clone(),
                    child_object: obj.clone(),
                })
            } else {
                None
            }
        })
    }
}

pub struct CommentParentResolver {
    cache: Cache<JsonnetASTGenerator>,
}

impl CommentParentResolver {
    fn handle_query(
        &self,
        cap: &QueryMatch,
        query: &Query,
        content: &str,
        node: Arc<Node>,
    ) -> Result<(Arc<Node>, tree_sitter::Point)> {
        // Due to the query these unwraps won't crash
        let comment = cap
            .captures
            .iter()
            .find(|c| c.index == query.capture_index_for_name("comment").expect("BUG"))
            .ok_or(anyhow!("No capture"))?;
        let object = cap
            .captures
            .iter()
            .find(|c| c.index == query.capture_index_for_name("object").expect("BUG"))
            .ok_or(anyhow!("No capture"))?;

        if !(LocationRange {
            file_name: node.node_base.loc_range.file_name.clone(),
            begin: object.node.start_position().into(),
            end: object.node.end_position().into(),
            ..Default::default()
        }
        .in_range(&node.node_base.loc_range.begin))
        {
            return Err(anyhow!("not in range"));
        }

        let re = regex::Regex::new(&format!(r#"^//\s*{}:\s*(?P<source>.+)$"#, MARKER_PREFIX))
            .expect("BUG: Wrong regex");
        // Check regex
        let node_name = comment
            .node
            .get_name(content)
            .ok_or(anyhow!("No node name"))?;
        let captures = re.captures(&node_name).ok_or(anyhow!("No captures"))?;
        // Get URL
        let source: String = captures["source"].parse()?;
        // Get yaml
        // TODO: async or thread. This will currently block everything!!
        // TODO: Add additional checks

        let mut hasher = Sha256::new();
        hasher.update(source.clone().as_bytes());
        let hash: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let cache_file = dirs::cache_dir()
            .ok_or(anyhow!("No cache dir"))?
            .join("grustonnet")
            .join(hash);
        let content = if !cache_file.exists() {
            let response = ureq::get(&source).call()?.body_mut().read_to_string()?;
            fs::create_dir_all(cache_file.parent().ok_or(anyhow!("no parent"))?)?;
            fs::write(&cache_file, &response)?;
            response
        } else {
            fs::read_to_string(&cache_file)?
        };
        let yaml_value: serde_json::Value = serde_yaml::from_str(&content)?;
        let json_value = serde_json::to_string(&yaml_value)?;
        let import_ast = self
            .cache
            .ast_generator
            .jsonnet
            .get_ast_snippet_binary("", &json_value)?;

        Ok((import_ast.into(), object.node.end_position()))
    }
}

impl ParentObjectResolver for CommentParentResolver {
    fn get_parent_object(&self, stack: &NodeStack) -> Option<ParentObjectInfo> {
        let uri = Uri::from_path(stack.peek()?.node_base.loc_range.file_name.clone()).ok()?;
        let doc = self.cache.get_document(&uri).ok()?;
        let tree = &doc.cst.clone()?;
        let ast = doc.ast?;
        let query_source = format!(
            r#"
            (
                (comment) @comment (#match? @comment "^//\\s*{}:.*")
                [
                    (object)
                    (member)
                ] @object
            )
            "#,
            MARKER_PREFIX
        );

        let query = Query::new(&tree.language(), &query_source).expect("BUG: Invalid query");
        let mut cursor = QueryCursor::new();
        let captures = cursor.matches(&query, tree.root_node(), doc.content.as_bytes());
        let mut result = None;
        let top_node = stack.peek()?;
        captures.for_each(|cap| {
            match self.handle_query(cap, &query, &doc.content, top_node.clone()) {
                Err(e) => {
                    log::debug!("Unable to get parent object: {e}");
                }
                Ok((obj, pos)) => {
                    let loc = Location::from(pos);
                    let stack = ast.get_stack_by_position(&loc);
                    if let Some(top_stack) = stack.peek()
                        && let NodeKind::DesugaredObject(top_object) = top_stack.node_kind.as_ref()
                    {
                        result = Some(ParentObjectInfo {
                            parent_object: obj,
                            child_object: top_object.clone(),
                        })
                    }
                }
            }
        });

        result
    }
}

impl<'a> GlobalObjectRef<'a> {
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>) -> Self {
        Self {
            cache,
            resolvers: Arc::new(RwLock::new(vec![
                Box::new(MarkerParentResolver {}),
                Box::new(CommentParentResolver {
                    cache: cache.clone(),
                }),
            ])),
        }
    }

    fn get_path_to_object(&self, stack: &NodeStack, obj: &DesugaredObject) -> Vec<String> {
        stack
            .stack
            .iter()
            .rev()
            .enumerate()
            .map_while(|(i, node)| {
                if let NodeKind::DesugaredObject(curr_obj) = node.node_kind.as_ref()
                    && curr_obj != obj
                    && let Some(next_object) = stack.peek_n(i + 1)
                    && let NodeKind::DesugaredObject(next_object) = next_object.node_kind.as_ref()
                {
                    Some(
                        next_object
                            .get_field_for_body_pos(&node.node_base.loc_range.begin)?
                            .get_name()?,
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_parent_object(&self, stack: &NodeStack) -> Option<(Vec<String>, DesugaredObject)> {
        let parent_info = self
            .resolvers
            .read_or_panic()
            .iter()
            .find_map(|resolver| resolver.get_parent_object(stack))?;
        let current_path = self.get_path_to_object(stack, &parent_info.child_object);
        // TODO: The stack is wrong here
        let resolved = resolve_node(self.cache, stack, parent_info.parent_object.clone()).ok()?;
        if let NodeKind::DesugaredObject(obj) = resolved.node_kind.as_ref() {
            Some((current_path.clone(), obj.clone()))
        } else {
            None
        }
    }
}

impl<'a> Completion for GlobalObjectRef<'a> {
    fn complete(&self, context: &CompletionContext) -> CompletionResult {
        let doc = self.cache.get_document(&context.uri)?;

        let stack = doc.get_ast()?.get_stack_by_position(&context.location);

        let Some((current_path, mut parent_object)) = self.get_parent_object(&stack) else {
            return Ok(CompletionList::default());
        };
        log::debug!("Object ref path {:?}", current_path);
        for path in current_path.iter().rev() {
            if let Some(found) = parent_object.get_field(path)
                && let NodeKind::DesugaredObject(new_obj) = found.body.node_kind.as_ref()
            {
                parent_object = new_obj.clone();
            } else {
                log::debug!(
                    "Unable to find {} in object. Names {:?}",
                    path,
                    parent_object
                        .fields
                        .iter()
                        .map(|f| f.get_name())
                        .collect::<Vec<_>>()
                );
                return Ok(CompletionList::default());
            }
        }
        // Look at the current object and build the path
        // Complete the object
        // Iterate over the parent object and skip all used fields
        Ok(CompletionList {
            is_incomplete: false,
            items: parent_object
                .fields
                .iter()
                .filter_map(|obj| {
                    // Get the target doc
                    // Extract the text from the position to insert
                    //let target_doc = self
                    //    .cache
                    //    .get_document(
                    //        &Uri::from_path(parent_field.1.node_base.loc_range.file_name).ok()?,
                    //    )
                    //    .ok()?;
                    Some(CompletionItem {
                        label: obj.get_name()?,
                        ..Default::default()
                    })
                })
                .collect(),
        })
    }
}
