use criterion::{Criterion, criterion_group, criterion_main};
use fsdr_blocks::type_converters::ScaledConverterBuilder;

pub fn bench_converters(c: &mut Criterion) {
    let n_samp = 65536;
    let input_i16: Vec<i16> = (0..n_samp)
        .map(|i| ((i as i32) % 65535 - 32768) as i16)
        .collect();
    let input_f32: Vec<f32> = (0..n_samp).map(|i| (i as f32 / 32768.0) - 1.0).collect();
    let input_u8: Vec<u8> = (0..n_samp).map(|i| (i % 256) as u8).collect();

    let mut group = c.benchmark_group("type_converters");
    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function("i16_to_f32_64k", |b| {
        b.iter(|| {
            let mut out = vec![0.0f32; n_samp];
            for (i, o) in input_i16.iter().zip(out.iter_mut()) {
                *o = ScaledConverterBuilder::<i16, f32>::convert(i);
            }
            out
        });
    });

    group.bench_function("f32_to_i16_64k", |b| {
        b.iter(|| {
            let mut out = vec![0i16; n_samp];
            for (i, o) in input_f32.iter().zip(out.iter_mut()) {
                *o = ScaledConverterBuilder::<f32, i16>::convert(i);
            }
            out
        });
    });

    group.bench_function("f32_to_i16_slice_simd_64k", |b| {
        let mut out = vec![0i16; n_samp];
        b.iter(|| {
            ScaledConverterBuilder::<f32, i16>::convert_slice(&input_f32, &mut out);
        });
    });

    group.bench_function("u8_to_f32_64k", |b| {
        b.iter(|| {
            let mut out = vec![0.0f32; n_samp];
            for (i, o) in input_u8.iter().zip(out.iter_mut()) {
                *o = ScaledConverterBuilder::<u8, f32>::convert(i);
            }
            out
        });
    });

    group.finish();
}

criterion_group!(benches, bench_converters);
criterion_main!(benches);
