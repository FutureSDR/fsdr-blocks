use crate::errors::SigMFError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AntennaExtension {
    #[serde(rename = "antenna:model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>, // Mandatory but required by the way we handle extension
    #[serde(rename = "antenna:type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

impl AntennaExtension {
    pub fn model(&self) -> Result<&String, SigMFError> {
        if let Some(model) = &self.model {
            return Ok(model);
        }
        Err(SigMFError::MissingMandatoryField("model"))
    }
}
