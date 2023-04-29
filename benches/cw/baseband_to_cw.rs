use criterion::{criterion_group, criterion_main, Criterion};
use fsdr_blocks::cw::baseband_to_cw::BaseBandToCW;
use fsdr_blocks::cw::shared::{char_to_baseband, CWAlphabet};
use futuresdr::runtime::Mocker;

// cargo bench --profile release --bench bb_to_cw
pub fn bench_baseband_to_cw(c: &mut Criterion) {
    let samples_per_dot = 1;
    let mut char_to_baseband_function = char_to_baseband(samples_per_dot);

    let message = "CQ CQ FutureSDR Community Blocks".to_uppercase();
    let baseband = message
        .chars()
        .flat_map(|c| char_to_baseband_function(&c))
        .collect::<Vec<f32>>();
    //println!("BaseBand Vector Length: {}, Content: {:?}", bb.len(), bb);

    let mut group = c.benchmark_group("baseband_to_cw");

    group.throughput(criterion::Throughput::Elements(baseband.len() as u64));

    group.bench_function(format!("mock-baseband-to-cw"), |b| {
        b.iter(|| {
            let block = BaseBandToCW::new_typed(100, samples_per_dot);
            let mut mocker = Mocker::new(block);

            mocker.input(0, baseband.clone());
            mocker.init_output::<CWAlphabet>(0, baseband.len());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_baseband_to_cw);
criterion_main!(benches);
