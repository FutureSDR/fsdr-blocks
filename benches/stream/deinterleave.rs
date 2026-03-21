use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fsdr_blocks::stream::Deinterleave;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};
use rand::RngExt;

pub fn deinterleave_f32(c: &mut Criterion) {
    let n_samp = 8192;
    let mut rng = rand::rng();
    let input: Vec<f32> = (0..n_samp).map(|_| rng.random()).collect();

    let mut group = c.benchmark_group("deinterleave");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("deinterleave_f32", |b| {
        b.iter(|| {
            let block: Deinterleave<f32, Reader<f32>, Writer<f32>, Writer<f32>> =
                Deinterleave::new();
            let mut mocker = Mocker::new(block);
            mocker.input().set(input.clone());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, deinterleave_f32);
criterion_main!(benches);
