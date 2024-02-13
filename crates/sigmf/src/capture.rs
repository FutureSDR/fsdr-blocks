use serde_json::Value;
use std::collections::HashMap;

#[cfg(feature = "quickcheck")]
use quickcheck::{empty_shrinker, Arbitrary, Gen};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Capture {
    #[serde(rename = "core:sample_start")]
    pub sample_start: Option<usize>,
    #[serde(rename = "core:global_index", skip_serializing_if = "Option::is_none")]
    pub global_index: Option<usize>,
    #[serde(rename = "core:frequency", skip_serializing_if = "Option::is_none")]
    pub frequency: Option<f64>,
    #[serde(rename = "core:datetime", skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
    #[serde(rename = "core:header_bytes", skip_serializing_if = "Option::is_none")]
    pub headers_bytes: Option<usize>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[cfg(feature = "quickcheck")]
impl Arbitrary for Capture {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut cap = Capture::default();
        for _ in 1..u8::arbitrary(g) {
            let key = String::arbitrary(g);
            let value = String::arbitrary(g);
            cap.extra.insert(key, Value::String(value));
        }
        cap
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        if self.extra.is_empty() {
            return empty_shrinker();
        }
        empty_shrinker() //TODO better
    }
}
