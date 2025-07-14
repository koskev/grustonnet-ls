use lsp_types::{SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend};
use name_variant::NamedVariant;
use strum::{EnumDiscriminants, EnumIter, IntoEnumIterator};

use crate::node::{
    location::LocationRange,
    types::{node::Node, node_kind::NodeKind},
};

macro_rules! token_enum {
    ($name: ident, $lsp_type: expr, $($item: expr),*) => {
        paste::paste! {
            #[derive(Default, Debug, EnumIter, EnumDiscriminants, NamedVariant, Clone)]
            pub enum $name {
                $(
                    $item,
                )*

                #[default]
                Unknown,
            }

            impl $name {
                pub fn to_vec() -> Vec<$lsp_type> {
                    $name::iter().map(|t| t.into()).collect()
                }

                fn to_int(&self) -> u32 {
                    [<$name Discriminants>]::from(self) as u32
                }
            }

            impl Into<$lsp_type> for $name {
                fn into(self) -> $lsp_type {
                    let name = self.variant_name();
                    let mut chars = name.chars();
                    let name = match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
                    };

                    $lsp_type::from(name)
                }
            }
        }
    };
}

token_enum!(
    SemanticToken,
    SemanticTokenType,
    // Enum Member
    Namespace,
    Type,
    Class,
    Enum,
    Interface,
    Struct,
    TypeParameter,
    Parameter,
    Variable,
    Property,
    EnumMember,
    Event,
    Function,
    Method,
    Macro,
    Keyword,
    Modifier,
    Comment,
    String,
    Number,
    Regexp,
    Operator
);

token_enum!(
    SemanticModifier,
    SemanticTokenModifier,
    // Enums
    Declaration,
    Definition,
    Readonly,
    Static,
    Deprecated,
    Abstract,
    Async,
    Modification,
    Documentation,
    DefaultLibrary
);

pub fn get_token_map() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: SemanticToken::to_vec(),
        token_modifiers: SemanticModifier::to_vec(),
        ..Default::default()
    }
}

#[derive(Default, Clone, Debug)]
struct SemanticData {
    node_type: SemanticToken,
    node_modifier: Vec<SemanticModifier>,
    location: LocationRange,
    length: u32,
}

#[derive(Default, Clone, Debug)]
struct SemanticDataList {
    data: Vec<SemanticData>,
}

impl Into<lsp_types::SemanticTokens> for SemanticDataList {
    fn into(mut self) -> lsp_types::SemanticTokens {
        let mut tokens = SemanticTokens::default();
        self.data.sort_by(|a, b| {
            let order = a.location.begin.line.cmp(&b.location.begin.line);
            if !order.is_eq() {
                return order;
            }
            a.location.begin.column.cmp(&b.location.begin.line)
        });
        for (i, data) in self.data.iter().enumerate() {
            let prev_token = if i == 0 {
                SemanticData::default()
            } else {
                self.data[i - 1].clone()
            };

            tokens.data.push(lsp_types::SemanticToken {
                delta_line: (data.location.begin.line - prev_token.location.begin.line) as u32,
                delta_start: if data.location.begin.line != prev_token.location.begin.line {
                    // Location starts at 1. As this is the only absolute value, we only need to
                    // substract 1 here
                    data.location.begin.column as u32 - 1
                } else {
                    (data.location.begin.column - prev_token.location.begin.column) as u32
                },
                length: data.length,
                token_type: data.node_type.to_int(),
                token_modifiers_bitset: data
                    .node_modifier
                    .iter()
                    .fold(0, |acc, val| acc | (1 << val.to_int())),
                ..Default::default()
            });
        }

        tokens
    }
}

pub fn get_tokens(root: &Node) -> SemanticTokens {
    let document_stack = root.get_complete_stack();

    let mut search_stack = document_stack.clone();
    let mut tokens = SemanticDataList::default();

    while let Some(current_node) = search_stack.stack.pop() {
        let location = &current_node.node_base.loc_range;
        match current_node.node_kind.as_ref() {
            NodeKind::Var(var) => {
                let mut data = SemanticData {
                    length: var.id.clone().unwrap_or_default().0.len() as u32,
                    node_type: SemanticToken::Variable,
                    location: location.clone(),
                    ..Default::default()
                };
                if var.id.clone().unwrap_or_default().0 == "std" {
                    data.node_modifier = vec![SemanticModifier::DefaultLibrary];
                } else if let Some(var_node) = var.resolve(&document_stack) {
                    match var_node.node_kind.as_ref() {
                        NodeKind::SelfNode => {
                            data.node_modifier = vec![SemanticModifier::DefaultLibrary];
                        }
                        NodeKind::Import(_import) => {
                            data.node_type = SemanticToken::Namespace;
                        }
                        _ => (),
                    }
                } else {
                    // We'll just assume all vars we can't resolve are params
                    // TODO: Get params on the stack
                    data.node_type = SemanticToken::Parameter;
                }
                tokens.data.push(data);
            }

            _ => (),
        };
    }

    tokens.into()
}
