use criterion::{Criterion, criterion_group, criterion_main};
use fsdr_blocks::math::FrequencyShifter;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};

pub fn bench_freq_shift(c: &mut Criterion) {
    let n_samp = 65536;
    let input_f32: Vec<f32> = vec![1.0; n_samp];
    let input_c32: Vec<Complex32> = vec![Complex32::new(1.0, 0.0); n_samp];

    let mut group = c.benchmark_group("freq_shift");
    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function("freq_shift_f32_64k", |b| {
        b.iter(|| {
            let block: FrequencyShifter<f32, Reader<f32>, Writer<f32>> =
                FrequencyShifter::new(1000.0, 48000.0);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input_f32.clone());
            mocker.run();
        });
    });

    group.bench_function("freq_shift_complex32_64k", |b| {
        b.iter(|| {
            let block: FrequencyShifter<Complex32, Reader<Complex32>, Writer<Complex32>> =
                FrequencyShifter::new(1000.0, 48000.0);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input_c32.clone());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_freq_shift);
criterion_main!(benches);
