use std::{
    path::PathBuf,
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
use tree_sitter::{Query, QueryCursor, QueryMatch, StreamingIterator};
use url::Url;
use utils::{RwLockPanic, cst::CstNodeHelper, uri::UriHelper};

use crate::{
    cache::JsonnetASTGenerator,
    completion::{
        global::object_ref::{file_handler::FileImporter, url_resolver::UrlNodeResolver},
        stdlib::functions::resolve_node,
    },
};

mod file_handler;
mod url_resolver;

const MARKER_PREFIX: &str = "grustonnetParentObject";

pub trait FileResolver {
    fn resolve_file(&self) -> Result<PathBuf>;
}

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
    max_download_size: u64,
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
            .captures()
            .iter()
            .find(|c| c.index == query.capture_index_for_name("comment").expect("BUG"))
            .ok_or(anyhow!("No capture"))?;
        let object = cap
            .captures()
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
        let captures = re
            .captures(&node_name)
            .ok_or(anyhow!("No captures in {}", node_name))?;
        // Get URL
        let source: String = captures["source"].parse()?;
        // Get yaml
        // TODO: async or thread. This will currently block everything!!
        // TODO: Add additional checks

        let source_url = Url::parse(&source);

        log::error!("Handling parent at {}", source);
        let mut source_file = match source_url {
            Ok(url) => UrlNodeResolver::new(url, self.max_download_size).resolve_file()?,
            Err(_e) => source.into(),
        };
        if source_file.is_relative() {
            let mut jpaths = self
                .cache
                .ast_generator
                .jsonnet
                .params
                .read_or_panic()
                .jpaths
                .clone();
            let parent_path = PathBuf::from(&node.node_base.loc_range.file_name.clone());
            let parent_str = parent_path
                .parent()
                .ok_or(anyhow!("File has no parent"))?
                .to_str()
                .unwrap_or_default();
            jpaths.insert(0, parent_str.to_string());

            source_file = jpaths
                .iter()
                .find_map(|path| {
                    let p = PathBuf::from(path).join(&source_file);
                    if p.exists() { Some(p) } else { None }
                })
                .ok_or(anyhow!("Unable to find file"))?;
        }

        let imported_node = FileImporter::new(self.cache.clone()).import_file(source_file)?;

        Ok((imported_node, object.node.end_position()))
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
                    (args (object))
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
                    log::warn!("Unable to get parent object: {e}");
                }
                Ok((obj, pos)) => {
                    log::debug!("Handling parent object!");
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
    pub fn new(cache: &'a Cache<JsonnetASTGenerator>, max_download_size: u64) -> Self {
        Self {
            cache,
            resolvers: Arc::new(RwLock::new(vec![
                Box::new(MarkerParentResolver {}),
                Box::new(CommentParentResolver {
                    cache: cache.clone(),
                    max_download_size,
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
