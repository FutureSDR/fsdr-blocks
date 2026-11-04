use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigMFError {
    #[error("Mandatory field is missing")]
    MissingMandatoryField(&'static str),
    #[error("JSON malformed or ")]
    JsonError(#[from] serde_json::Error),
    #[error("Unknown DatasetFormat")]
    UnknownDatasetFormat(String),
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("Sample rate must be positive and less than 1e250")]
    BadSampleRate(),
}
