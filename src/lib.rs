//! This library acts as a toolbox on top of [FutureSDR][`futuresdr`] to easily build your own flowgraph.
//! It is made by the community for the community.

// #![feature(async_fn_in_trait)]

#[macro_use]
pub extern crate async_trait;

#[cfg(feature = "crossbeam")]
pub mod channel;

#[cfg(feature = "async-channel")]
pub mod async_channel;

#[cfg(feature = "cw")]
pub mod cw;

pub mod math;
pub mod sigmf;
pub mod stdinout;
pub mod stream;
pub mod type_converters;

pub mod serde_pmt;
