use crate::{errors::SigMFError, AntennaExtension, DatasetFormat, Extension};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Global {
    #[serde(rename = "core:datatype")]
    pub datatype: Option<DatasetFormat>, // It is mandatory but we want to be lax in parsing
    #[serde(rename = "core:version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>, // It is mandatory but we want to be lax in parsing
    #[serde(rename = "core:sample_rate", skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<f64>,
    #[serde(rename = "core:num_channels", skip_serializing_if = "Option::is_none")]
    pub num_channels: Option<usize>,
    #[serde(rename = "core:sha512", skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
    #[serde(rename = "core:offset", skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(rename = "core:description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "core:author", skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(rename = "core:meta_doi", skip_serializing_if = "Option::is_none")]
    pub meta_doi: Option<String>,
    #[serde(rename = "core:data_doi", skip_serializing_if = "Option::is_none")]
    pub data_doi: Option<String>,
    #[serde(rename = "core:recorder", skip_serializing_if = "Option::is_none")]
    pub recorder: Option<String>,
    #[serde(rename = "core:license", skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(rename = "core:hw", skip_serializing_if = "Option::is_none")]
    pub hw: Option<String>,
    #[serde(rename = "core:collection", skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(rename = "core:metadata_only", skip_serializing_if = "Option::is_none")]
    pub metadata_only: Option<bool>,
    #[serde(rename = "core:dataset", skip_serializing_if = "Option::is_none")]
    pub dataset: Option<String>,
    #[serde(
        rename = "core:trailing_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_bytes: Option<usize>,
    #[serde(rename = "core:extensions", skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<Extension>>,
    #[serde(flatten)]
    pub antenna: AntennaExtension,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl Global {
    pub fn version(&self) -> Result<&String, SigMFError> {
        if let Some(version) = &self.version {
            return Ok(version);
        }
        Err(SigMFError::MissingMandatoryField("version"))
    }

    pub fn datatype(&self) -> Result<&DatasetFormat, SigMFError> {
        if let Some(datatype) = &self.datatype {
            return Ok(datatype);
        }
        Err(SigMFError::MissingMandatoryField("datatype"))
    }
}

impl Default for Global {
    fn default() -> Self {
        Self {
            datatype: Some(DatasetFormat::Cf32Le),
            version: Some("1.0.0".to_string()),
            sample_rate: None,
            num_channels: None,
            sha512: None,
            offset: None,
            description: None,
            author: None,
            meta_doi: None,
            data_doi: None,
            recorder: None,
            license: None,
            hw: None,
            collection: None,
            metadata_only: None,
            dataset: None,
            trailing_bytes: None,
            extensions: None,
            antenna: AntennaExtension::default(),
            extra: HashMap::new(),
        }
    }
}
