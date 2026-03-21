# SigMF - fsdr-blocks

## Overview
The `sigmf` crate (in `crates/sigmf`) provides a Rust implementation of the [Signal Metadata Format](https://github.com/sigmf/SigMF).

## Structure
- **Global:** Top-level metadata about the recording.
- **Captures:** Segment-specific metadata (sample rate, frequency).
- **Annotations:** Time/frequency-indexed labels.

## Usage in fsdr-blocks
- `SigmfSource`: Reads `.sigmf-meta` and `.sigmf-data` files into a FutureSDR flowgraph.
- `SigmfSink`: Records flowgraph data into SigMF-compliant files.

## Extensions
Supports SigMF extensions (e.g., `AntennaExtension`). New extensions should be added as modules in `crates/sigmf/src/`.
