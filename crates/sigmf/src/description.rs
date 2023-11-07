use crate::{Global, Annotation, Capture, Collection, SigMFError, DatasetFormat};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Description {
    #[serde(rename = "global", skip_serializing_if = "Option::is_none")]
    pub global: Option<Global>,
    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
    #[serde(rename = "captures", skip_serializing_if = "Option::is_none")]
    pub captures: Option<Vec<Capture>>,
    #[serde(rename = "collections", skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<Collection>>,
}

impl Description {
    pub fn global(&self) -> Result<&Global, SigMFError> {
        if let Some(global) = &self.global {
            return Ok(&global)
        }
        return Err(SigMFError::MissingMandatoryField("global"))
    }

    pub fn annotations(&self) -> Result<&Vec<Annotation>, SigMFError> {
        if let Some(annotations) = &self.annotations {
            return Ok(&annotations)
        }
        return Err(SigMFError::MissingMandatoryField("annotations"))
    }

    pub fn captures(&self) -> Result<&Vec<Capture>, SigMFError> {
        if let Some(captures) = &self.captures {
            return Ok(&captures)
        }
        return Err(SigMFError::MissingMandatoryField("captures"))
    }
}

impl Default for Description {
    fn default() -> Self {
        Self {
            global: Some(Global::default()),
            annotations: Some(Vec::new()),
            captures: Some(Vec::new()),
            collections: Some(Vec::new()),
        }
    }
}

pub struct DescriptionBuilder(Description);

impl Default for DescriptionBuilder {
    fn default() -> Self {
        Self(Description::default())
    }
}

// impl  DescriptionBuilder {
//     pub fn Datatype(datatype: &str) -> DescriptionBuilder {

//     }
// }

impl From<DatasetFormat> for DescriptionBuilder {
    fn from(value: DatasetFormat) -> Self {
        let mut desc = DescriptionBuilder::default();
        desc.0.global = Some(Global { datatype: Some(value), version: Some("1.0.0".to_string())});
        desc
    }
}