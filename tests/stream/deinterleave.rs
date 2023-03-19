use fsdr_blocks::stream::*;
use futuresdr::anyhow::Result;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[test]
fn deinterleave_u8() -> Result<()> {
    let mut fg = Flowgraph::new();

    let deinterleaver = Deinterleave::<u8>::new();

    let orig: Vec<u8> = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink_0 = VectorSinkBuilder::<u8>::new().build();
    let vect_sink_1 = VectorSinkBuilder::<u8>::new().build();

    connect!(fg,
        src > deinterleaver;
        deinterleaver.out0 > vect_sink_0;
        deinterleaver.out1 > vect_sink_1;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<u8>>(vect_sink_0).unwrap();
    let snk_0 = snk_0.items();

    let snk_1 = fg.kernel::<VectorSink<u8>>(vect_sink_1).unwrap();
    let snk_1 = snk_1.items();

    assert_eq!(snk_0.len(), orig.len() / 2);
    assert_eq!(snk_0.len(), snk_1.len());
    assert!(snk_0.iter().all(|v| *v == 0));
    assert!(snk_1.iter().all(|v| *v == 1));

    Ok(())
}
