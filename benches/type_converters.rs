use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fsdr_blocks::type_converters::TypeConverter;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};
use rand::RngExt;

pub fn scale_convert_u8_f32(c: &mut Criterion) {
    let n_samp = 8192;
    let mut rng = rand::rng();
    let input: Vec<u8> = (0..n_samp).map(|_| rng.random()).collect();

    let mut group = c.benchmark_group("type_converters");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("scale_convert_u8_f32", |b| {
        b.iter(|| {
            let block: TypeConverter<u8, f32, Reader<u8>, Writer<f32>> = TypeConverter::new(true);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input.clone());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, scale_convert_u8_f32);
criterion_main!(benches);
