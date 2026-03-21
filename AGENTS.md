# AGENTS.md - Hub for fsdr-blocks

Building blocks for FutureSDR signal processing library for SDR and real-time DSP.

## Tech Stack
- **Language:** Rust (Edition 2024)
- **Core Library:** [FutureSDR](https://www.futuresdr.org)
- **Serialization:** Serde, custom PMT (Polymorphic Types)
- **Testing:** Cargo test, QuickCheck, Criterion (benchmarks)

## Critical Commands
- **Install:** `cargo build`
- **Lint:** `./check.sh` (Runs fmt, clippy, and tests with all features)
- **Test:** `cargo test --all-features`
- **Bench:** `cargo bench --all-features`

## Documentation Index
- [Architecture](agent_docs/architecture.md): **Trigger:** Designing new blocks or understanding flowgraph connectivity.
- [Conventions](agent_docs/conventions.md): **Trigger:** Before writing any code to ensure alignment with Rust 2024 and FutureSDR idioms.
- [SDR & DSP](agent_docs/sdr_dsp.md): **Trigger:** Modifying signal processing logic, gain control, or frequency shifts.
- [SigMF](agent_docs/sigmf.md): **Trigger:** Working with Signal Metadata Format (SigMF) recordings or collections.

## Verification Loop
You MUST run `./check.sh` and ensure all tests pass before declaring a task "done."
Always verify that your changes didn't break conditional feature flags (`crossbeam`, `async-channel`, `cw`).
