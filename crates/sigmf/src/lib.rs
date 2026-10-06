//! Rust implementation of the Signal Metadata Format (SigMF) specification.
//!
//! ## Overview
//! Provides data structures and helper utilities to parse, create, and serialize
//! SigMF recording metadata (`.sigmf-meta`).
//!
//! - **Global:** Top-level metadata about the recording.
//! - **Captures:** Segment-specific metadata (sample rate, frequency).
//! - **Annotations:** Time/frequency-indexed labels.

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

mod errors;
pub use errors::SigMFError;

mod annotation;
pub use annotation::Annotation;

mod antenna_extension;
pub use antenna_extension::AntennaExtension;

mod capture;
pub use capture::Capture;

mod collection;
pub use collection::Collection;

mod dataset_format;
pub use dataset_format::{DatasetFormat, DatasetFormatBuilder};

mod description;
pub use description::{Description, DescriptionBuilder};

mod extension;
pub use extension::Extension;

mod global;
pub use global::Global;

mod recording;
pub use recording::{Recording, RecordingBuilder};
