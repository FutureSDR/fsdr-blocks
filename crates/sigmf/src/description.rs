use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

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

    pub fn to_writer<W>(&self, writer: W) -> Result<(), SigMFError>
    where
        W: io::Write,
    {
        Ok(serde_json::to_writer(writer, self)?)
    }

    pub fn create<P>(&self, path: P) -> Result<(), SigMFError>
    where
        P: AsRef<Path>,
    {
        let f = File::create(path)?;
        self.to_writer(f)
    }

    pub fn open<P>(path: P) -> Result<Description, SigMFError>
    where
        P: AsRef<Path>,
    {
        let meta_file = File::open(path)?;
        let rdr = BufReader::new(meta_file);
        let desc: Result<Description, serde_json::Error> = serde_json::from_reader(rdr);
        Ok(desc?)
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

    pub fn open<P>(path: P) -> Result<DescriptionBuilder, SigMFError>
    where
        P: AsRef<Path>,
    {
        let desc = Description::open(path)?;
        Ok(DescriptionBuilder(desc))
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
