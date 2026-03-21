use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fsdr_blocks::math::FrequencyShifter;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};
use rand::RngExt;

pub fn freq_shift_c32(c: &mut Criterion) {
    let n_samp = 8192;
    let mut rng = rand::rng();
    let input: Vec<Complex32> = (0..n_samp)
        .map(|_| Complex32::new(rng.random(), rng.random()))
        .collect();

    let mut group = c.benchmark_group("math");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("freq_shift_c32", |b| {
        b.iter(|| {
            let block: FrequencyShifter<Complex32, Reader<Complex32>, Writer<Complex32>> =
                FrequencyShifter::new(2000.0, 48000.0);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input.clone());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, freq_shift_c32);
criterion_main!(benches);
