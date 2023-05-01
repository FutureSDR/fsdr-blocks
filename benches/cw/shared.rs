use criterion::{criterion_group, criterion_main, Criterion};
use fsdr_blocks::cw::shared::{char_to_baseband, msg_to_cw};

// cargo bench --profile release --bench shared --features="cw"
pub fn bench_char_to_baseband(c: &mut Criterion) {
    let samples_per_dot = 16384;
    let mut char_to_baseband_function = char_to_baseband(samples_per_dot);

    let message = "CQ CQ FutureSDR Community Blocks".to_uppercase();
    let bb = message
        .chars()
        .flat_map(|c| char_to_baseband_function(&c))
        .collect::<Vec<f32>>();
    //println!("BaseBand Vector Length: {}, Content: {:?}", bb.len(), bb);

    let mut group = c.benchmark_group("char_to_baseband");

    group.throughput(criterion::Throughput::Elements(bb.len() as u64));

    group.bench_function(format!("char_to_bb"), |b| {
        b.iter(|| {
            message
                .chars()
                .flat_map(|c| char_to_baseband_function(&c))
                .for_each(drop);
        });
    });

    group.finish();
}

// cargo bench --profile release --bench shared --features="cw"
pub fn bench_msg_to_cw(c: &mut Criterion) {
    let message = "CQ CQ FutureSDR Community Blocks"
        .to_uppercase()
        .chars()
        .collect::<Vec<char>>();
    let msg_slice = message.as_slice();
    //println!("Message chars Vector Length: {}, Content: {:?}", msg_slice.len(), msg_slice);

    let mut group = c.benchmark_group("msg_to_cw");

    group.throughput(criterion::Throughput::Elements(msg_slice.len() as u64));

    group.bench_function(format!("msg_to_cw"), |b| {
        b.iter(|| {
            msg_to_cw(msg_slice);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_char_to_baseband, bench_msg_to_cw);
criterion_main!(benches);
