use crate::errors::SigMFError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Global {
    #[serde(rename = "core:datatype")] //try_from = "FromType")
    pub datatype: Option<String>, // It is mandatory but we want to be lax in parsing
    #[serde(rename = "core:version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Global {
    pub fn version(&self) -> Result<&String, SigMFError> {
        if let Some(version) = &self.version {
            return Ok(&version)
        }
        return Err(SigMFError::MissingMandatoryField("version"))
    }

    pub fn datatype(&self) -> Result<&String, SigMFError> {
        if let Some(datatype) = &self.datatype {
            return Ok(&datatype)
        }
        return Err(SigMFError::MissingMandatoryField("datatype"))
    }
}