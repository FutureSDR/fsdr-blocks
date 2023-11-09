use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigMFError {
    #[error("Mandatory field is missing")]
    MissingMandatoryField(&'static str),
    #[error("JSON malformed or ")]
    JsonError(#[from] serde_json::Error),
    #[error("Unknown DatasetFormat")]
    UnknownDatasetFormat(String),
}
