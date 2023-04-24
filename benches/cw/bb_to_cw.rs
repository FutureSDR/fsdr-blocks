use criterion::{criterion_group, criterion_main, Criterion};
use fsdr_blocks::cw::bb_to_cw::BBToCW;
use fsdr_blocks::cw::shared::{char_to_bb, CWAlphabet};
use futuresdr::runtime::Mocker;

// cargo bench --profile release --bench bb_to_cw
pub fn bench_bb_to_cw(c: &mut Criterion) {
    let samples_per_dot = 1;
    let mut char_to_bb_function = char_to_bb(samples_per_dot);

    let message = "CQ CQ FutureSDR Community Blocks".to_uppercase();
    let bb = message
        .chars()
        .flat_map(|c| char_to_bb_function(&c))
        .collect::<Vec<f32>>();
    //println!("BaseBand Vector Length: {}, Content: {:?}", bb.len(), bb);

    let mut group = c.benchmark_group("crossbeam_sink");

    group.throughput(criterion::Throughput::Elements(bb.len() as u64));

    group.bench_function(format!("mock-bb-to-cw"), |b| {
        b.iter(|| {
            let block = BBToCW::new_typed(100, samples_per_dot);
            let mut mocker = Mocker::new(block);

            mocker.input(0, bb.clone());
            mocker.init_output::<CWAlphabet>(0, bb.len());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_bb_to_cw);
criterion_main!(benches);
