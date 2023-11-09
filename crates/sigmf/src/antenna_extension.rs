use crate::errors::SigMFError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AntennaExtension {
    #[serde(rename = "antenna:model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>, // Mandatory but we want to be laxed
}

impl AntennaExtension {
    pub fn model(&self) -> Result<&String, SigMFError> {
        if let Some(model) = &self.model {
            return Ok(&model);
        }
        return Err(SigMFError::MissingMandatoryField("model"));
    }
}

impl Default for AntennaExtension {
    fn default() -> Self {
        Self { model: None }
    }
}
