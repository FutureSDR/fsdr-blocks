use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fsdr_blocks::cw::cw_to_char::CWToChar;
use fsdr_blocks::cw::shared::{get_alphabet, msg_to_cw};
use futuresdr::runtime::Mocker;

// cargo bench --profile release --bench cw_to_char
pub fn bench_cw_to_char(c: &mut Criterion) {
    let message = "CQ CQ FutureSDR Community Blocks"
        .to_uppercase()
        .chars()
        .collect::<Vec<char>>();
    let cw = msg_to_cw(message.as_slice());
    //println!("CW-Alphabet Vector Length: {}, Content: {:?}", cw.len(), cw);

    let mut group = c.benchmark_group("cw_to_char");

    group.throughput(criterion::Throughput::Elements(cw.len() as u64));

    for i in 1..4 {
        group.bench_with_input(BenchmarkId::new("work", i), &i, |b, i| {
            b.iter(|| {
                let block = CWToChar::new_typed(get_alphabet(), *i);
                let mut mocker = Mocker::new(block);

                mocker.input(0, cw.clone());
                mocker.init_output::<char>(0, cw.len());
                mocker.run();
            })
        });
    }

    /*group.bench_function(format!("mock-cw-to-char-1"), |b| {
        b.iter(|| {
            let block = CWToChar::new_typed(get_alphabet(), 1);
            let mut mocker = Mocker::new(block);

            mocker.input(0, cw.clone());
            mocker.init_output::<char>(0, cw.len());
            mocker.run();
        });
    });*/

    group.finish();
}

criterion_group!(benches, bench_cw_to_char);
criterion_main!(benches);
