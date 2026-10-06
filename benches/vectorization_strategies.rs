#![feature(portable_simd)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::simd::prelude::*;

// -----------------------------------------------------------------------------
// 1. DEINTERLEAVE (I/Q Split) STRATEGIES
// -----------------------------------------------------------------------------

/// Strategy 1: Naive scalar loop with boolean branching per sample (original performance killer)
#[inline(never)]
fn deinterleave_naive_branch(input: &[f32], out_i: &mut [f32], out_q: &mut [f32]) {
    let mut first = true;
    let mut i_idx = 0;
    let mut q_idx = 0;
    for &val in input {
        if first {
            out_i[i_idx] = val;
            i_idx += 1;
        } else {
            out_q[q_idx] = val;
            q_idx += 1;
        }
        first = !first;
    }
}

/// Strategy 2: Autovectorizable chunked slice processing (chunks_exact 2)
#[inline(never)]
fn deinterleave_chunks_autovec(input: &[f32], out_i: &mut [f32], out_q: &mut [f32]) {
    let pairs = (input.len() / 2).min(out_i.len()).min(out_q.len());
    let in_chunks = &input[..pairs * 2];
    let i_slice = &mut out_i[..pairs];
    let q_slice = &mut out_q[..pairs];

    for (chunk, (i_out, q_out)) in in_chunks
        .chunks_exact(2)
        .zip(i_slice.iter_mut().zip(q_slice.iter_mut()))
    {
        *i_out = chunk[0];
        *q_out = chunk[1];
    }
}

/// Strategy 3: Unrolled 8-sample batch autovectorization loop
#[inline(never)]
fn deinterleave_unrolled8_autovec(input: &[f32], out_i: &mut [f32], out_q: &mut [f32]) {
    let pairs = (input.len() / 2).min(out_i.len()).min(out_q.len());
    let len_8 = pairs / 4;

    for k in 0..len_8 {
        let in_idx = k * 8;
        let out_idx = k * 4;
        out_i[out_idx + 0] = input[in_idx + 0];
        out_q[out_idx + 0] = input[in_idx + 1];
        out_i[out_idx + 1] = input[in_idx + 2];
        out_q[out_idx + 1] = input[in_idx + 3];
        out_i[out_idx + 2] = input[in_idx + 4];
        out_q[out_idx + 2] = input[in_idx + 5];
        out_i[out_idx + 3] = input[in_idx + 6];
        out_q[out_idx + 3] = input[in_idx + 7];
    }

    // Remainder
    for k in (len_8 * 4)..pairs {
        out_i[k] = input[k * 2];
        out_q[k] = input[k * 2 + 1];
    }
}

/// Strategy 4: Explicit std::simd portable SIMD
#[inline(never)]
fn deinterleave_std_simd(input: &[f32], out_i: &mut [f32], out_q: &mut [f32]) {
    let pairs = (input.len() / 2).min(out_i.len()).min(out_q.len());
    let (in_chunks, in_rem) = input[..pairs * 2].as_chunks::<8>();
    let (i_chunks, i_rem) = out_i[..pairs].as_chunks_mut::<4>();
    let (q_chunks, q_rem) = out_q[..pairs].as_chunks_mut::<4>();

    for ((chunk, i_out), q_out) in in_chunks
        .iter()
        .zip(i_chunks.iter_mut())
        .zip(q_chunks.iter_mut())
    {
        let v = Simd::from_array(*chunk);
        let i_vec = simd_swizzle!(v, [0, 2, 4, 6]);
        let q_vec = simd_swizzle!(v, [1, 3, 5, 7]);
        *i_out = i_vec.to_array();
        *q_out = q_vec.to_array();
    }

    for ((chunk, i_out), q_out) in in_rem
        .chunks_exact(2)
        .zip(i_rem.iter_mut())
        .zip(q_rem.iter_mut())
    {
        *i_out = chunk[0];
        *q_out = chunk[1];
    }
}

/// Strategy 5: fearless_simd safe dynamic vectorization
#[inline(never)]
fn deinterleave_fearless_simd(input: &[f32], out_i: &mut [f32], out_q: &mut [f32]) {
    fearless_simd::dispatch!(fearless_simd::Level::new(), _simd => {
        let pairs = (input.len() / 2).min(out_i.len()).min(out_q.len());
        let in_slice = &input[..pairs * 2];
        let i_slice = &mut out_i[..pairs];
        let q_slice = &mut out_q[..pairs];

        for (chunk, (i_out, q_out)) in in_slice
            .chunks_exact(2)
            .zip(i_slice.iter_mut().zip(q_slice.iter_mut()))
        {
            *i_out = chunk[0];
            *q_out = chunk[1];
        }
    });
}

// -----------------------------------------------------------------------------
// 2. FREQUENCY SHIFT / COMPLEX VECTOR MULTIPLICATION STRATEGIES
// -----------------------------------------------------------------------------

#[derive(Clone, Copy)]
#[repr(C)]
struct Complex32 {
    re: f32,
    im: f32,
}

impl Complex32 {
    fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
}

/// Strategy 1: Naive scalar complex multiply
#[inline(never)]
fn freq_shift_naive_scalar(input: &[Complex32], phasors: &[Complex32], output: &mut [Complex32]) {
    let len = input.len().min(phasors.len()).min(output.len());
    for i in 0..len {
        let a = input[i];
        let b = phasors[i];
        let re = a.re * b.re - a.im * b.im;
        let im = a.re * b.im + a.im * b.re;
        output[i] = Complex32::new(re, im);
    }
}

/// Strategy 2: Rust 1.98 Algebraic Float Math methods (algebraic_mul, algebraic_sub, algebraic_add)
#[inline(never)]
fn freq_shift_algebraic_autovec(
    input: &[Complex32],
    phasors: &[Complex32],
    output: &mut [Complex32],
) {
    let len = input.len().min(phasors.len()).min(output.len());
    let in_slice = &input[..len];
    let ph_slice = &phasors[..len];
    let out_slice = &mut output[..len];

    for ((a, b), out) in in_slice
        .iter()
        .zip(ph_slice.iter())
        .zip(out_slice.iter_mut())
    {
        let re_re = f32::algebraic_mul(a.re, b.re);
        let im_im = f32::algebraic_mul(a.im, b.im);
        let re_im = f32::algebraic_mul(a.re, b.im);
        let im_re = f32::algebraic_mul(a.im, b.re);

        let re = f32::algebraic_sub(re_re, im_im);
        let im = f32::algebraic_add(re_im, im_re);
        *out = Complex32::new(re, im);
    }
}

/// Strategy 3: Explicit std::simd batch complex multiplication (SOA format)
#[inline(never)]
fn freq_shift_std_simd_soa(
    input_re: &[f32],
    input_im: &[f32],
    phasor_re: &[f32],
    phasor_im: &[f32],
    out_re: &mut [f32],
    out_im: &mut [f32],
) {
    let len = input_re
        .len()
        .min(input_im.len())
        .min(phasor_re.len())
        .min(out_re.len());
    let (in_re_chunks, _) = input_re[..len].as_chunks::<8>();
    let (in_im_chunks, _) = input_im[..len].as_chunks::<8>();
    let (ph_re_chunks, _) = phasor_re[..len].as_chunks::<8>();
    let (ph_im_chunks, _) = phasor_im[..len].as_chunks::<8>();
    let (out_re_chunks, _) = out_re[..len].as_chunks_mut::<8>();
    let (out_im_chunks, _) = out_im[..len].as_chunks_mut::<8>();

    for i in 0..in_re_chunks.len() {
        let a_re = Simd::from_array(in_re_chunks[i]);
        let a_im = Simd::from_array(in_im_chunks[i]);
        let b_re = Simd::from_array(ph_re_chunks[i]);
        let b_im = Simd::from_array(ph_im_chunks[i]);

        let re_re = a_re * b_re;
        let im_im = a_im * b_im;
        let re_im = a_re * b_im;
        let im_re = a_im * b_re;

        let res_re = re_re - im_im;
        let res_im = re_im + im_re;

        out_re_chunks[i] = res_re.to_array();
        out_im_chunks[i] = res_im.to_array();
    }
}

/// Strategy 4: fearless_simd dynamic dispatch for complex multiplication
#[inline(never)]
fn freq_shift_fearless_simd(input: &[Complex32], phasors: &[Complex32], output: &mut [Complex32]) {
    fearless_simd::dispatch!(fearless_simd::Level::new(), _simd => {
        let len = input.len().min(phasors.len()).min(output.len());
        for i in 0..len {
            let a = input[i];
            let b = phasors[i];
            let re = f32::algebraic_sub(f32::algebraic_mul(a.re, b.re), f32::algebraic_mul(a.im, b.im));
            let im = f32::algebraic_add(f32::algebraic_mul(a.re, b.im), f32::algebraic_mul(a.im, b.re));
            output[i] = Complex32::new(re, im);
        }
    });
}

// -----------------------------------------------------------------------------
// 3. TYPE CONVERTER (f32 -> i16 scale & clamp) STRATEGIES
// -----------------------------------------------------------------------------

/// Strategy 1: Naive scalar element conversion
#[inline(never)]
fn convert_naive_scalar(input: &[f32], output: &mut [i16]) {
    let len = input.len().min(output.len());
    for i in 0..len {
        let scaled = input[i] * 32767.0;
        output[i] = scaled.round().clamp(-32768.0, 32767.0) as i16;
    }
}

/// Strategy 2: Fast-math / algebraic scalar conversion
#[inline(never)]
fn convert_algebraic_autovec(input: &[f32], output: &mut [i16]) {
    let len = input.len().min(output.len());
    let in_slice = &input[..len];
    let out_slice = &mut output[..len];

    for (&s, d) in in_slice.iter().zip(out_slice.iter_mut()) {
        let scaled = f32::algebraic_mul(s, 32767.0);
        *d = scaled.round().clamp(-32768.0, 32767.0) as i16;
    }
}

/// Strategy 3: std::simd batch conversion
#[inline(never)]
fn convert_std_simd(input: &[f32], output: &mut [i16]) {
    let len = input.len().min(output.len());
    let (src_chunks, src_rem) = input[..len].as_chunks::<8>();
    let (dst_chunks, dst_rem) = output[..len].as_chunks_mut::<8>();

    let scale = Simd::splat(32767.0f32);
    let min_val = Simd::splat(-32768.0f32);
    let max_val = Simd::splat(32767.0f32);

    for (s, d) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
        let v = Simd::from_array(*s);
        let scaled = v * scale;
        let clamped = scaled.simd_clamp(min_val, max_val);
        let arr = clamped.to_array();
        for i in 0..8 {
            d[i] = arr[i].round() as i16;
        }
    }

    for (&s, d) in src_rem.iter().zip(dst_rem.iter_mut()) {
        let scaled = s * 32767.0;
        *d = scaled.round().clamp(-32768.0, 32767.0) as i16;
    }
}

/// Strategy 4: fearless_simd type conversion
#[inline(never)]
fn convert_fearless_simd(input: &[f32], output: &mut [i16]) {
    fearless_simd::dispatch!(fearless_simd::Level::new(), _simd => {
        let len = input.len().min(output.len());
        let in_slice = &input[..len];
        let out_slice = &mut output[..len];

        for (&s, d) in in_slice.iter().zip(out_slice.iter_mut()) {
            let scaled = f32::algebraic_mul(s, 32767.0);
            *d = scaled.round().clamp(-32768.0, 32767.0) as i16;
        }
    });
}

// -----------------------------------------------------------------------------
// BENCHMARK GROUPS
// -----------------------------------------------------------------------------

fn bench_deinterleave_strategies(c: &mut Criterion) {
    let n_samp = 65536; // 64K f32 samples
    let input: Vec<f32> = (0..n_samp).map(|x| (x as f32) * 0.001).collect();
    let mut out_i = vec![0.0f32; n_samp / 2];
    let mut out_q = vec![0.0f32; n_samp / 2];

    let mut group = c.benchmark_group("vectorization_deinterleave_64k");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("1_naive_branch", |b| {
        b.iter(|| deinterleave_naive_branch(&input, &mut out_i, &mut out_q));
    });

    group.bench_function("2_chunks_autovec", |b| {
        b.iter(|| deinterleave_chunks_autovec(&input, &mut out_i, &mut out_q));
    });

    group.bench_function("3_unrolled8_autovec", |b| {
        b.iter(|| deinterleave_unrolled8_autovec(&input, &mut out_i, &mut out_q));
    });

    group.bench_function("4_std_simd", |b| {
        b.iter(|| deinterleave_std_simd(&input, &mut out_i, &mut out_q));
    });

    group.bench_function("5_fearless_simd", |b| {
        b.iter(|| deinterleave_fearless_simd(&input, &mut out_i, &mut out_q));
    });

    group.finish();
}

fn bench_freq_shift_strategies(c: &mut Criterion) {
    let n_samp = 65536;
    let input: Vec<Complex32> = (0..n_samp)
        .map(|x| Complex32::new((x as f32) * 0.001, (x as f32) * 0.002))
        .collect();
    let phasors: Vec<Complex32> = (0..n_samp)
        .map(|x| Complex32::new(((x as f32) * 0.01).cos(), ((x as f32) * 0.01).sin()))
        .collect();
    let mut output = vec![Complex32::new(0.0, 0.0); n_samp];

    let in_re: Vec<f32> = input.iter().map(|c| c.re).collect();
    let in_im: Vec<f32> = input.iter().map(|c| c.im).collect();
    let ph_re: Vec<f32> = phasors.iter().map(|c| c.re).collect();
    let ph_im: Vec<f32> = phasors.iter().map(|c| c.im).collect();
    let mut out_re = vec![0.0f32; n_samp];
    let mut out_im = vec![0.0f32; n_samp];

    let mut group = c.benchmark_group("vectorization_freq_shift_64k");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("1_naive_scalar", |b| {
        b.iter(|| freq_shift_naive_scalar(&input, &phasors, &mut output));
    });

    group.bench_function("2_algebraic_autovec", |b| {
        b.iter(|| freq_shift_algebraic_autovec(&input, &phasors, &mut output));
    });

    group.bench_function("3_std_simd_soa", |b| {
        b.iter(|| {
            freq_shift_std_simd_soa(&in_re, &in_im, &ph_re, &ph_im, &mut out_re, &mut out_im)
        });
    });

    group.bench_function("4_fearless_simd", |b| {
        b.iter(|| freq_shift_fearless_simd(&input, &phasors, &mut output));
    });

    group.finish();
}

fn bench_type_converter_strategies(c: &mut Criterion) {
    let n_samp = 65536;
    let input: Vec<f32> = (0..n_samp)
        .map(|x| ((x % 200) as f32 / 100.0) - 1.0)
        .collect();
    let mut output = vec![0i16; n_samp];

    let mut group = c.benchmark_group("vectorization_converter_64k");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("1_naive_scalar", |b| {
        b.iter(|| convert_naive_scalar(&input, &mut output));
    });

    group.bench_function("2_algebraic_autovec", |b| {
        b.iter(|| convert_algebraic_autovec(&input, &mut output));
    });

    group.bench_function("3_std_simd", |b| {
        b.iter(|| convert_std_simd(&input, &mut output));
    });

    group.bench_function("4_fearless_simd", |b| {
        b.iter(|| convert_fearless_simd(&input, &mut output));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_deinterleave_strategies,
    bench_freq_shift_strategies,
    bench_type_converter_strategies
);
criterion_main!(benches);
