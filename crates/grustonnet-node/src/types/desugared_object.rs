use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use bincode::{Decode, Encode};
use itertools::Itertools;
use jsonnet_bridge::go::{ASTBridge, ASTBridgeImpl, FormatOptions};
use jsonnet_location::{Location, LocationRange};
use serde::{Deserialize, Serialize};

use crate::types::{local_bind::LocalBind, node::Node, node_kind::NodeKind};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct DesugaredObjectField {
    pub name: Arc<Node>,
    pub body: Arc<Node>,
    pub loc_range: LocationRange,
    pub hide: i32,
    pub plus_super: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase", tag = "T")]
pub struct DesugaredObject {
    pub asserts: Vec<Node>,
    pub fields: Vec<DesugaredObjectField>,
    pub locals: Vec<LocalBind>,
}
impl DesugaredObjectField {
    pub fn get_name(&self) -> Option<String> {
        match self.name.node_kind.as_ref() {
            NodeKind::LiteralString(name) => Some(name.value.clone()),
            _ => None,
        }
    }
}

impl Display for DesugaredObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let body = self
            .fields
            .iter()
            .map(|f| {
                format!(
                    "{} : {}",
                    f.get_name().unwrap_or_default(),
                    f.body.node_kind.get_value().unwrap_or_default()
                )
            })
            .collect::<String>();
        // TODO: use the actual format options
        let formatted = ASTBridgeImpl::format_snippet(
            "".into(),
            format!("{{ {} }}", body),
            FormatOptions::default(),
        );
        if !formatted.error_data.is_empty() {
            write!(f, "{{ {} }}", body)
        } else {
            let formatted_string = String::from_utf8(formatted.ast_data).unwrap_or_default();
            write!(f, "{}", formatted_string)
        }
    }
}

impl DesugaredObject {
    pub fn merge(&self, other: &DesugaredObject) -> DesugaredObject {
        let mut new_object = self.clone();
        log::debug!("Merging {} and {}", self, other);
        new_object.asserts.extend(other.asserts.clone());
        new_object.fields.extend(other.fields.clone());
        new_object.fields = new_object
            .fields
            .iter()
            .chain(other.fields.iter())
            .unique_by(|o| o.get_name())
            .cloned()
            .collect();
        new_object.locals.extend(other.locals.clone());

        new_object
    }

    pub fn get_name_at(&self, pos: &Location) -> Option<String> {
        self.fields.iter().find_map(|field| {
            if field.loc_range.in_range(pos) {
                field.get_name()
            } else {
                None
            }
        })
    }

    pub fn get_field(&self, name: &str) -> Option<&DesugaredObjectField> {
        self.fields.iter().find(|field| {
            if let Some(field_name) = field.get_name() {
                field_name == name
            } else {
                false
            }
        })
    }
}
