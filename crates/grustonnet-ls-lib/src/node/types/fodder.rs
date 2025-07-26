use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
pub struct Fodder(pub Vec<FodderElement>);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
pub struct FodderElement {
    pub comment: Vec<String>,
    pub kind: i32,
    pub blanks: i32,
    pub indent: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Decode, Encode)]
#[serde(rename_all = "PascalCase")]
pub enum FodderKind {
    #[default]
    FodderLineEnd,
    FodderInterstitial,
    FodderParagraph,
}
