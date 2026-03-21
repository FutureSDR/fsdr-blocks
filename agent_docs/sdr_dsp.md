# SDR & DSP - fsdr-blocks

## Signal Processing Idioms
- **Complex Numbers:** Use `num_complex::Complex32` (or generic `T: ComplexFloat`).
- **Buffers:** Always use `input.slice()` and `output.slice()` in the `work` function.
- **Consumption/Production:** Explicitly call `input.consume(n)` and `output.produce(n)` after processing.

## Key Blocks
- **AGC (Automatic Gain Control):** Implements a feedback loop to maintain target power. See `src/agc.rs`.
- **FreqShift:** Performs digital down-conversion/up-conversion. See `src/math/freq_shift.rs`.
- **Type Converters:** Crucial for translating between raw bytes and SDR-specific types.

## Math Operations
- **Prefer Explicit SIMD:** Use `std::simd` and specialization (see `DeinterleaveSupported`) for performance-critical blocks in hot loops. This avoids dependency on brittle compiler autovectorization.
- **Precision:** Be mindful of floating-point precision and squelch thresholds.
- **Error Accumulation:** Periodically re-calculate phase in recurrence relations to prevent drift (e.g., in `FreqShift`).
