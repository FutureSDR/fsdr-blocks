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
- Prefer vectorized operations where possible, though many blocks currently use iterator-based processing for the compiler to auto-vectorize.
- Be mindful of floating-point precision and squelch thresholds.
