use crate::Recording;
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

#[cfg(feature = "quickcheck")]
use quickcheck::{empty_shrinker, single_shrinker, Arbitrary, Gen};

use crate::{Annotation, Capture, Collection, DatasetFormat, Extension, Global, SigMFError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Description {
    #[serde(rename = "global", skip_serializing_if = "Option::is_none")]
    pub global: Option<Global>,
    #[serde(rename = "captures", skip_serializing_if = "Option::is_none")]
    pub captures: Option<Vec<Capture>>,
    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
    #[serde(rename = "collection", skip_serializing_if = "Option::is_none")]
    pub collection: Option<Collection>,
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

    pub fn annotations_mut(&mut self) -> Result<&mut Vec<Annotation>, SigMFError> {
        if let Some(annotations) = &mut self.annotations {
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

    pub fn captures_mut(&mut self) -> Result<&Vec<Capture>, SigMFError> {
        if let Some(captures) = &mut self.captures {
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

    pub fn to_writer_pretty<W>(&self, writer: W) -> Result<(), SigMFError>
    where
        W: io::Write,
    {
        Ok(serde_json::to_writer_pretty(writer, self)?)
    }

    pub fn create<P>(&self, path: P) -> Result<(), SigMFError>
    where
        P: AsRef<Path>,
    {
        let f = File::create(path)?;
        self.to_writer(f)
    }

    pub fn create_pretty<P>(&self, path: P) -> Result<(), SigMFError>
    where
        P: AsRef<Path>,
    {
        let f = File::create(path)?;
        self.to_writer_pretty(f)
    }

    pub fn from_reader<R>(reader: R) -> Result<Description, SigMFError>
    where
        R: io::Read,
    {
        let desc: Result<Description, serde_json::Error> = serde_json::from_reader(reader);
        Ok(desc?)
    }

    pub fn open<P>(path: P) -> Result<Description, SigMFError>
    where
        P: AsRef<Path>,
    {
        let meta_file = File::open(path)?;
        let rdr = BufReader::new(meta_file);
        Description::from_reader(rdr)
    }
}

impl Default for Description {
    fn default() -> Self {
        Self {
            global: Some(Global::default()),
            annotations: Some(Vec::new()),
            captures: Some(Vec::new()),
            collection: None,
        }
    }
}

#[cfg(feature = "quickcheck")]
impl Arbitrary for Description {
    fn arbitrary(g: &mut Gen) -> Self {
        let global = Global::arbitrary(g);
        let mut desc = DescriptionBuilder::from(global);
        if bool::arbitrary(g) {
            let caps = Vec::<Capture>::arbitrary(g);
            desc.captures(caps);
        }
        desc.build()
            .expect("arbitrary shall build valid description")
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        if *self == Description::default() {
            return empty_shrinker();
        }
        empty_shrinker()
    }
}

#[derive(Debug, Default)]
pub struct DescriptionBuilder(Description);

impl DescriptionBuilder {
    pub fn collection() -> DescriptionBuilder {
        DescriptionBuilder(Description {
            collection: Some(Collection::default()),
            global: None,
            captures: None,
            annotations: None,
        })
    }

    pub fn sample_rate(&mut self, sample_rate: f64) -> Result<&mut DescriptionBuilder, SigMFError> {
        if sample_rate.is_nan() || !(0.0..=1e251).contains(&sample_rate) {
            return Err(SigMFError::BadSampleRate());
        }
        let global = self.0.global.as_mut().unwrap();
        global.sample_rate = Some(sample_rate);
        Ok(self)
    }

    pub fn extension(
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

    pub fn captures(&mut self, captures: Vec<Capture>) -> &mut DescriptionBuilder {
        self.0.captures = Some(captures);
        self
    }

    pub fn build(&self) -> Result<Description, SigMFError> {
        // TODO checks for mandatory fields
        Ok(self.0.clone())
    }

    pub fn open<P>(path: P) -> Result<DescriptionBuilder, SigMFError>
    where
        P: AsRef<Path>,
    {
        let desc = Description::open(path)?;
        Ok(DescriptionBuilder(desc))
    }

    pub fn add_stream(&mut self, stream: Recording) -> Result<&mut Self, SigMFError> {
        self.0
            .collection
            .as_mut()
            .expect("")
            .streams
            .as_mut()
            .expect("msg")
            .push(stream);
        Ok(self)
    }
}

impl From<DatasetFormat> for DescriptionBuilder {
    fn from(value: DatasetFormat) -> Self {
        let mut desc = DescriptionBuilder::default();
        let global = Global {
            datatype: Some(value),
            ..Default::default()
        };
        desc.0.global = Some(global);
        desc
    }
}

impl From<Global> for DescriptionBuilder {
    fn from(value: Global) -> Self {
        let mut desc = DescriptionBuilder::default();
        desc.0.global = Some(value);
        desc
    }
}
