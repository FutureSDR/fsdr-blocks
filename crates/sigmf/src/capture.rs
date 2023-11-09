use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capture {
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
