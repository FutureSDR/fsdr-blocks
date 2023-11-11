use crate::errors::SigMFError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AntennaExtension {
    #[serde(rename = "antenna:model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>, // Mandatory but we want to be laxed
}

impl AntennaExtension {
    pub fn model(&self) -> Result<&String, SigMFError> {
        if let Some(model) = &self.model {
            return Ok(model);
        }
        Err(SigMFError::MissingMandatoryField("model"))
    }
}
