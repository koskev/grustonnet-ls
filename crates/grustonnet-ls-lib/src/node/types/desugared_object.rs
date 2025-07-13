use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::node::{
    location::{Location, LocationRange},
    types::{local_bind::LocalBind, node::Node, node_kind::NodeKind},
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub struct DesugaredObjectField {
    pub name: Arc<Node>,
    pub body: Arc<Node>,
    pub loc_range: LocationRange,
    pub hide: i32,
    pub plus_super: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase", tag = "Type")]
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
        let names: String = self
            .fields
            .iter()
            .map(|f| format!("field {}", f.name.node_kind))
            .collect();
        write!(f, "{}", names)
    }
}

impl DesugaredObject {
    pub fn merge(&self, other: &DesugaredObject) -> DesugaredObject {
        let mut new_object = self.clone();
        log::debug!("Merging {} and {}", self, other);
        new_object.asserts.extend(other.asserts.clone());
        new_object.fields.extend(other.fields.clone());
        new_object.locals.extend(other.locals.clone());

        new_object
    }

    pub fn get_name_at(&self, pos: &Location) -> Option<String> {
        self.fields.iter().find_map(|field| {
            if field.loc_range.in_range(&pos) {
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
