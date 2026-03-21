# Conventions - fsdr-blocks

## Rust Standards
- **Edition:** 2024. Use modern idioms (e.g., `async fn` in traits, let-else).
- **Formatting:** Strictly adhere to `cargo fmt`.

## Block Implementation Boilerplate
Every block should typically include:
1. **Struct Definition:** Use `#[derive(Block)]` and specify `#[message_inputs(...)]` if applicable.
2. **Implementation:** A `new` method and message handler methods (returning `Result<Pmt>`).
3. **Kernel Trait:** Implement `async fn work` to handle the data processing loop.
4. **Builder Pattern:** Use a `BlockBuilder` struct for complex configuration (see `src/agc.rs`).

## Performance Acceleration
For blocks in hot loops (AGC, Frequency Shift, Deinterleave), prefer explicit SIMD over compiler-dependent autovectorization:
1. **Feature Flags:** Ensure `portable_simd` and `specialization` are enabled in `src/lib.rs`.
2. **Specialization Pattern:** Define a `*Supported` trait (e.g., `DeinterleaveSupported`) with a `default` scalar implementation and specialized SIMD implementations for `f32`, `u8`, `i8`, `i16`.
3. **Macro Reuse:** Use macros to implement SIMD logic across different types to avoid code duplication.
4. **Benchmarking:** Every accelerated block MUST have a corresponding `Criterion` benchmark in `benches/`.

## Error Handling
- Use `futuresdr::anyhow::Result` for block operations.
- Prefer `Context` from `anyhow` for descriptive error messages in I/O operations.

## Feature Management
- Use `#[cfg(feature = "...")]` for blocks that depend on optional crates like `crossbeam-channel` or `async-channel`.
- Always test with `--all-features` to ensure no regressions in optional components.

## Testing
- **Unit Tests:** Located in `tests/`.
- **Property-Based Testing:** Use `quickcheck` for robust validation of DSP algorithms.
- **Benchmarks:** Located in `benches/`, using `Criterion`.
