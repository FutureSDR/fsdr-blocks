use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fsdr_blocks::stream::Deinterleave;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};

pub fn bench_deinterleave(c: &mut Criterion) {
    let n_samp = 65536;
    let input: Vec<f32> = (0..n_samp).map(|x| x as f32).collect();

    let mut group = c.benchmark_group("deinterleave");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("deinterleave_f32_64k", |b| {
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

criterion_group!(benches, bench_deinterleave);
criterion_main!(benches);
