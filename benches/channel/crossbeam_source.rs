use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use futuresdr::runtime::Mocker;
use fsdr_blocks::channel::CrossbeamSource;

/// This benchmark seems to highly depend on the underlying scheduling of polling from the channel
pub fn crossbeam_source_boxed_slice_u32(c: &mut Criterion) {
    let n_samp = 8192;
    let input: Vec<u32> = rand::thread_rng()
        .sample_iter(rand::distributions::Uniform::<u32>::new(0, 1024))
        .take(n_samp)
        .collect();

    let (tx, rx) = crossbeam_channel::unbounded::<Box<[u32]>>();

    let mut group = c.benchmark_group("crossbeam_source");

    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function(format!("mock-u32-crossbeam-source"), |b| {
        b.iter(|| {
            let block = CrossbeamSource::new_typed(rx.clone());
            let mut mocker = Mocker::new(block);

            tx.try_send(input.clone().into_boxed_slice()).unwrap();

            mocker.init_output::<u32>(0, n_samp);
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, crossbeam_source_boxed_slice_u32);
criterion_main!(benches);
