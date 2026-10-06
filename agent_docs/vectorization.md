# Vectorization & SIMD Optimization Guide - fsdr-blocks

This document details the vectorization strategy, performance findings, and micro-optimization idioms used across `fsdr-blocks`.

---

## 1. Core Principles for SIMD & Autovectorization

To ensure peak throughput across heterogeneous CPU targets (x86-64 AVX2/AVX-512, ARM NEON/SVE, Apple Silicon, WASM SIMD128):

1. **Branchless Loop Iteration:**
   - **Never** place conditional scalar branches (`if`/`else` boolean toggles) inside sample-by-sample processing loops.
   - Use slice chunking (`.chunks_exact(N)` or `.as_chunks::<N>()`) to present contiguous, branch-free memory to LLVM.
   - *Impact:* Eliminating branches increased deinterleaving performance by **8x** (from 1.76 Gelem/s to 14.35 Gelem/s).

2. **Safe Float Reassociation & FMA (`f32::algebraic_*` + `mul_add`):**
   - Rust 1.98 introduced `f32::algebraic_add`, `algebraic_mul`, `algebraic_sub`, and `algebraic_div`. These functions inject LLVM `reassoc` flags in 100% Safe Rust without unstable intrinsics or `#![feature(core_intrinsics)]`.
   - Combine with `a.mul_add(b, c)` (or `x.mul_add(y, z.algebraic_mul(w))`) to emit fused multiply-add hardware instructions (`vfmadd` / `vfmsub`) on AVX2/NEON.

3. **Memory Layout: Structure of Arrays (SOA) vs. Array of Structures (AOS):**
   - Interleaved complex structures (`[Complex32 { re, im }]`) introduce shuffle/permute overhead during SIMD vectorization.
   - Separate real and imaginary streams (`I: &[f32]`, `Q: &[f32]`) unlock **+23% higher SIMD compute throughput** (up to 4.29 Gelem/s) by enabling direct, un-shuffled vector loads.

---

## 2. Benchmark Findings & Comparison

Evaluated on native architecture (`RUSTFLAGS="-C target-cpu=native"`) across 65,536 sample blocks:

| DSP Workload | Naive Implementation | Optimized (Autovec + Algebraic + FMA) | Peak Throughput | Speedup |
| :--- | :--- | :--- | :--- | :--- |
| **Deinterleave (IQ Split)** | 37.18 µs (Branching) | **4.56 µs** (`chunks_exact(2)`) | 14.35 Gelem/s | **8.15x ⚡** |
| **FreqShift (Complex Mul)** | 18.73 µs (AOS) | **15.27 µs** (SOA + SIMD FMA) | 4.29 Gelem/s | **1.23x 🚀** |
| **AGC Gain Loop** | 5.15 µs (Unfused) | **4.98 µs** (`mul_add` FMA) | 13.15 Gelem/s | **1.03x** |
| **Type Converter (`u8` ➔ `f32`)** | 5.18 µs (Scalar) | **4.92 µs** (`mul_add` FMA) | 13.32 Gelem/s | **1.05x** |

---

## 3. Implementation Idioms for Block Authors

### A. Fused Multiply-Add (FMA) in Gain & Power Loops
```rust
// Compute squared magnitude in a single FMA cycle
let power = re.mul_add(re, im.algebraic_mul(im));

// Update feedback gain loop
gain = error.mul_add(adjustment_rate, gain).clamp(0.0, max_gain);
```

### B. Complex Multiplication
```rust
// Fast complex product using FMA for imaginary term
let re = f32::algebraic_sub(
    f32::algebraic_mul(a.re, b.re),
    f32::algebraic_mul(a.im, b.im),
);
let im = a.re.mul_add(b.im, a.im.algebraic_mul(b.re));
```
