use fsdr_blocks::stream::*;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Result;
use futuresdr::runtime::Runtime;

#[test]
fn deinterleave_u8() -> Result<()> {
    let mut fg = Flowgraph::new();

    let deinterleaver = Deinterleave::<u8>::new();

    let orig: Vec<u8> = (0..100).map(|i| (i % 2) as u8).collect();
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink_0 = VectorSink::<u8>::new(1024);
    let vect_sink_1 = VectorSink::<u8>::new(1024);

    connect!(fg,
        src > deinterleaver;
        deinterleaver.out0 > vect_sink_0;
        deinterleaver.out1 > vect_sink_1;
    );
    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get()?;
    let snk_0 = snk_0.items();

    let snk_1 = vect_sink_1.get()?;
    let snk_1 = snk_1.items();

    assert_eq!(snk_0.len(), 50);
    assert_eq!(snk_1.len(), 50);
    assert!(snk_0.iter().all(|v| *v == 0));
    assert!(snk_1.iter().all(|v| *v == 1));

    Ok(())
}

#[test]
fn deinterleave_odd_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let deinterleaver = Deinterleave::<f32>::new();

    let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let src = VectorSource::<f32>::new(orig.clone());
    let vect_sink_0 = VectorSink::<f32>::new(1024);
    let vect_sink_1 = VectorSink::<f32>::new(1024);

    connect!(fg,
        src > deinterleaver;
        deinterleaver.out0 > vect_sink_0;
        deinterleaver.out1 > vect_sink_1;
    );
    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get()?;
    let snk_0 = snk_0.items();

    let snk_1 = vect_sink_1.get()?;
    let snk_1 = snk_1.items();

    assert_eq!(snk_0.len(), 3);
    assert_eq!(snk_1.len(), 2);
    assert_eq!(snk_0, &[0.0, 2.0, 4.0]);
    assert_eq!(snk_1, &[1.0, 3.0]);

    Ok(())
}
