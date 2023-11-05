use crate::sigmf::{Global, Annotation, Capture, Collection};
use futuresdr::anyhow::{Result, anyhow};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Description {
    #[serde(rename = "global", skip_serializing_if = "Option::is_none")]
    pub global: Option<Global>,
    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
    #[serde(rename = "captures", skip_serializing_if = "Option::is_none")]
    pub captures: Option<Vec<Capture>>,
    #[serde(rename = "collections", skip_serializing_if = "Option::is_none")]
    pub collections: Option<Collection>,
}

impl Description {
    pub fn global(&self) -> Result<&Global> {
        if let Some(global) = &self.global {
            return Ok(&global)
        }
        return Err(anyhow!("global is unset (while being mandatory)"))
    }
}