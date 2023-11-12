use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(rename = "core:sample_start")]
    pub sample_start: Option<usize>,
    #[serde(rename = "core:sample_count", skip_serializing_if = "Option::is_none")]
    pub sample_count: Option<usize>,
    #[serde(
        rename = "core:freq_lower_edge",
        skip_serializing_if = "Option::is_none"
    )]
    pub freq_lower_edge: Option<f64>,
    #[serde(
        rename = "core:freq_upper_edge",
        skip_serializing_if = "Option::is_none"
    )]
    pub freq_upper_edge: Option<f64>,
    #[serde(rename = "core:label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "core:generator", skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    #[serde(rename = "core:comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "core:uuid", skip_serializing_if = "Option::is_none")]
    pub uuid: Option<uuid::Uuid>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
