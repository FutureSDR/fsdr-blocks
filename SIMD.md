# SIMD Optimization Roadmap - fsdr-blocks

This document outlines the prioritized blocks for SIMD refactoring based on their impact in typical SDR pipelines (e.g., `csdr` style flows).

## Prioritization Matrix

| Priority | Block | File | Rationale |
| :--- | :--- | :--- | :--- |
| **High** | `convert_u8_f` | `src/type_converters.rs` | Entry point for raw samples; high throughput, simple math. |
| **High** | `FreqShift` | `src/math/freq_shift.rs` | Per-sample complex rotation; solves loop-carried dependencies. |
| **Medium** | `FmDemod` | TBD | Quadrature math (cross-products) is "heavy" and benefits from vectorization. |
| **Medium** | `BinarySlicer` | TBD | Comparisons are perfect for SIMD masks and bit-packing. |
| **Low** | `Deinterleave` | `src/stream/deinterleave.rs` | **COMPLETED.** Memory-bound but established the pattern. |

## Implementation Pattern: SIMD + Specialization

To maintain compatibility and maximize performance, we use the following pattern:

1.  **Nightly Features:** Enable `#![feature(portable_simd)]` and `#![feature(specialization)]`.
2.  **Specialization Trait:** Define a `*Supported` trait (e.g., `TypeConvertSupported`).
3.  **Default Logic:** Provide a `default` scalar implementation for all types.
4.  **SIMD Specialization:** Implement specialized SIMD versions for `f32`, `u8`, `i16`, etc.
5.  **Generic Helpers:** Use internal `_scalar_logic` and `_simd_logic` functions to minimize duplication.
6.  **Macros:** Employ `macro_rules!` to apply the same SIMD logic across multiple types (e.g., `u8`, `i8`).

## Completed Optimizations

### 1. Deinterleave (`src/stream/deinterleave.rs`)
- **Status:** Done.
- **Impact:** ~6% gain on `f32` (memory-bound).
- **Pattern:** `DeinterleaveSupported` trait with macro-based SIMD implementations.
