use criterion::{Criterion, criterion_group, criterion_main};
use fsdr_blocks::agc::Agc;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};
use rand::RngExt;

pub fn bench_agc_f32(c: &mut Criterion) {
    let n_samp = 65536;
    let input: Vec<f32> = rand::rng()
        .sample_iter(rand::distr::Uniform::<f32>::new(-2.0, 2.0).unwrap())
        .take(n_samp)
        .collect();

    let mut group = c.benchmark_group("agc");
    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function("agc_f32_64k", |b| {
        b.iter(|| {
            let block: Agc<f32, Reader<f32>, Writer<f32>> =
                Agc::new(0.0, 10.0, 1.0, 0.01, 1.0, false, false);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input.clone());
            mocker.run();
        });
    });

    #[allow(clippy::chunks_exact_to_as_chunks)]
    let input_c32: Vec<Complex32> = input
        .chunks_exact(2)
        .map(|chunk| Complex32::new(chunk[0], chunk[1]))
        .collect();

    group.throughput(criterion::Throughput::Elements(input_c32.len() as u64));
    group.bench_function("agc_complex32_32k", |b| {
        b.iter(|| {
            let block: Agc<Complex32, Reader<Complex32>, Writer<Complex32>> =
                Agc::new(0.0, 10.0, 1.0, 0.01, 1.0, false, false);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input_c32.clone());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_agc_f32);
criterion_main!(benches);
