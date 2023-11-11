use crate::{Annotation, Capture, Collection, DatasetFormat, Extension, Global, SigMFError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Description {
    #[serde(rename = "global", skip_serializing_if = "Option::is_none")]
    pub global: Option<Global>,
    #[serde(rename = "captures", skip_serializing_if = "Option::is_none")]
    pub captures: Option<Vec<Capture>>,
    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
    #[serde(rename = "collections", skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<Collection>>,
}

impl Description {
    pub fn global(&self) -> Result<&Global, SigMFError> {
        if let Some(global) = &self.global {
            return Ok(global);
        }
        Err(SigMFError::MissingMandatoryField("global"))
    }

    pub fn global_mut(&mut self) -> Result<&mut Global, SigMFError> {
        if let Some(global) = &mut self.global {
            return Ok(global);
        }
        Err(SigMFError::MissingMandatoryField("global"))
    }

    pub fn annotations(&self) -> Result<&Vec<Annotation>, SigMFError> {
        if let Some(annotations) = &self.annotations {
            return Ok(annotations);
        }
        Err(SigMFError::MissingMandatoryField("annotations"))
    }

    pub fn captures(&self) -> Result<&Vec<Capture>, SigMFError> {
        if let Some(captures) = &self.captures {
            return Ok(captures);
        }
        Err(SigMFError::MissingMandatoryField("captures"))
    }
}

impl Default for Description {
    fn default() -> Self {
        Self {
            global: Some(Global::default()),
            annotations: Some(Vec::new()),
            captures: Some(Vec::new()),
            collections: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct DescriptionBuilder(Description);

impl DescriptionBuilder {
    pub fn sample_rate(&mut self, sample_rate: f64) -> &mut DescriptionBuilder {
        let global = self.0.global.as_mut().unwrap();
        global.sample_rate = Some(sample_rate);
        self
    }

    pub fn extensions(
        &mut self,
        name: &str,
        version: &str,
        optional: bool,
    ) -> &mut DescriptionBuilder {
        let global = self.0.global.as_mut().unwrap();
        let new_ext = Extension {
            name: name.to_string(),
            version: version.to_string(),
            optional,
        };
        if let Some(extensions) = &mut global.extensions {
            extensions.push(new_ext);
        } else {
            global.extensions = Some(vec![new_ext]);
        }
        self
    }

    pub fn build(&self) -> Result<Description, SigMFError> {
        // TODO checks
        Ok(self.0.clone())
    }
}

impl From<DatasetFormat> for DescriptionBuilder {
    fn from(value: DatasetFormat) -> Self {
        let mut desc = DescriptionBuilder::default();
        let mut global = Global::default();
        global.datatype = Some(value);
        desc.0.global = Some(global);
        desc
    }
}
