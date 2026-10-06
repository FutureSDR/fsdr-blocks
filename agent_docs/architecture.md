# Architecture - fsdr-blocks

This project follows the **Flowgraph/Block** paradigm provided by FutureSDR.

## Core Concepts
- **Blocks:** The fundamental units of processing. Defined using `#[derive(Block)]`.
- **Flowgraph:** A directed acyclic graph (usually) where blocks are nodes and streams/messages are edges.
- **Kernels:** The execution logic of a block. Most blocks in this repository are CPU-based and implement the `Kernel` trait.

## Data Movement
- **Streams:** High-throughput data (e.g., IQ samples) passed via `CpuBufferReader`/`CpuBufferWriter`.
- **Messages:** Asynchronous control signals or metadata passed as `Pmt` (Polymorphic Types).

## Dependency on FutureSDR
This project is tightly coupled with `futuresdr`. It uses a local path dependency in `Cargo.toml` by default (`../FutureSDR`), which indicates it is often developed alongside the core library.

## Block Types
- **Sources/Sinks:** Handle I/O (e.g., `StdinSink`, `SigmfSource`).
- **Processing Blocks:** Transform data (e.g., `Agc`, `FreqShift`).
- **Adapters:** Connect different async runtimes or channel types (e.g., `CrossbeamSink`).
