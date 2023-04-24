use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use futuresdr::runtime::Mocker;
use fsdr_blocks::channel::CrossbeamSink;

/// This benchmark seems to highly depend on the underlying scheduling of polling from the channel
pub fn crossbeam_sink_boxed_slice_u32(c: &mut Criterion) {
    let n_samp = 8192;
    let input: Vec<u32> = rand::thread_rng()
        .sample_iter(rand::distributions::Uniform::<u32>::new(0, 1024))
        .take(n_samp)
        .collect();
    let input = input.into_boxed_slice();
    let input = vec![input];

    let (tx, rx) = crossbeam_channel::unbounded::<Box<[u32]>>();

    let mut group = c.benchmark_group("crossbeam_sink");

    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function(format!("mock-u32-crossbeam-sink"), |b| {
        b.iter(|| {
            let block = CrossbeamSink::new_typed(tx.clone());
            let mut mocker = Mocker::new(block);

            mocker.input(0, input.clone());
            mocker.run();

            // receive again all samples sent into the crossbeam_sink...
            rx.iter().take(1).for_each(drop);
        });
    });

    group.finish();
}


criterion_group!(benches, crossbeam_sink_boxed_slice_u32);
criterion_main!(benches);
