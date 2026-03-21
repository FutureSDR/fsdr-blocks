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
