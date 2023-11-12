use crate::{Extension, Recording};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    #[serde(rename = "core:version")]
    pub version: Option<String>,
    #[serde(rename = "core:description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "core:author", skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(
        rename = "core:collection_doi",
        skip_serializing_if = "Option::is_none"
    )]
    pub collection_doi: Option<String>,
    #[serde(rename = "core:license", skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(rename = "core:extensions", skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<Extension>>,
    #[serde(rename = "core:streams", skip_serializing_if = "Option::is_none")]
    pub streams: Option<Vec<Recording>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
