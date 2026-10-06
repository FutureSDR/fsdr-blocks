# SigMF Crate

Rust implementation of the [Signal Metadata Format (SigMF)](https://github.com/sigmf/SigMF) specification for `fsdr-blocks`.

## Overview
The `sigmf` crate provides data structures and helpers to parse, build, and serialize SigMF metadata (`.sigmf-meta`) files.

## Structure
- **Global:** Top-level metadata about the recording (description, dataset format, author, etc.).
- **Captures:** Segment-specific metadata (sample rate, center frequency, datetime).
- **Annotations:** Time/frequency-indexed labels and region-of-interest annotations.

## Usage in fsdr-blocks
The main `fsdr-blocks` library provides flowgraph blocks utilizing this crate:
- `SigMFSource`: Reads `.sigmf-meta` and `.sigmf-data` files into a FutureSDR flowgraph.
- `SigMFSink`: Records flowgraph data into SigMF-compliant files.

## Extensions
Supports SigMF extensions (e.g., `AntennaExtension`). New extensions should be added as modules in `crates/sigmf/src/`.

## Useful Links
* [SigMF Specification](https://github.com/sigmf/SigMF)
* [IQEngine](https://www.iqengine.org/)
* [libsigmf](https://github.com/sigmf/libsigmf/)
* [gr-sigmf](https://github.com/skysafe/gr-sigmf)