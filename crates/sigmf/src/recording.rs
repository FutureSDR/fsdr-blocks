use crate::{Description, SigMFError};
use sha2::{Digest, Sha512};
use std::io::Read;
use std::path::Path;
use std::{fs::File, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    #[serde(rename = "name")]
    pub name: Option<PathBuf>,
    #[serde(rename = "hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl Recording {
    pub fn hash(&self) -> Result<&String, SigMFError> {
        if let Some(hash) = &self.hash {
            return Ok(hash);
        }
        Err(SigMFError::MissingMandatoryField("hash"))
    }

    pub fn sigmf_data(&mut self) -> Result<&Path, SigMFError> {
        if let Some(basename) = &mut self.name {
            basename.set_extension("sigmf-data");
            return Ok(basename.as_path());
        }
        Err(SigMFError::MissingMandatoryField("name"))
    }

    pub fn sigmf_meta(&mut self) -> Result<&Path, SigMFError> {
        if let Some(basename) = &mut self.name {
            basename.set_extension("sigmf-meta");
            return Ok(basename.as_path());
        }
        Err(SigMFError::MissingMandatoryField("name"))
    }

    pub fn compute_sha512(&mut self) -> Result<String, SigMFError> {
        let path = self.sigmf_data()?;
        let mut data_file = File::open(path)?;
        let mut hasher = Sha512::new();
        let mut buffer = [0; 1024];

        loop {
            let count = data_file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let computed_sha512 = hasher.finalize();
        let computed_sha512 = hex::encode(computed_sha512);
        Ok(computed_sha512)
    }

    pub fn load_description(&mut self) -> Result<Description, SigMFError> {
        let path = self.sigmf_meta()?;
        let desc = Description::open(path)?;
        Ok(desc)
    }
}

pub struct RecordingBuilder(Recording);

impl From<PathBuf> for RecordingBuilder {
    fn from(value: PathBuf) -> Self {
        RecordingBuilder(Recording {
            name: Some(value),
            hash: None,
        })
    }
}

impl From<&PathBuf> for RecordingBuilder {
    fn from(value: &PathBuf) -> Self {
        RecordingBuilder(Recording {
            name: Some(value.to_path_buf()),
            hash: None,
        })
    }
}

impl From<&Path> for RecordingBuilder {
    fn from(value: &Path) -> Self {
        RecordingBuilder(Recording {
            name: Some(value.to_path_buf()),
            hash: None,
        })
    }
}

impl RecordingBuilder {
    pub fn build(&self) -> Recording {
        self.0.clone()
    }

    /// Load the .sigmf-meta file and copy the sha512 hash if any
    pub fn load_description(&mut self) -> Result<(Self, Description), SigMFError> {
        let desc = self.0.load_description()?;
        let mut new_hash = self.0.hash.clone();
        if let Some(hash) = &desc.global()?.sha512 {
            new_hash = Some((*hash).clone());
        }
        let res = RecordingBuilder(Recording {
            name: self.0.name.clone(),
            hash: new_hash,
        });
        Ok((res, desc))
    }

    /// Load the .sigmf-data and compute the sha512 hash
    pub fn compute_sha512(&mut self) -> Result<&mut Self, SigMFError> {
        let hash = self.0.compute_sha512()?;
        self.0.hash = Some(hash);
        Ok(self)
    }
}
