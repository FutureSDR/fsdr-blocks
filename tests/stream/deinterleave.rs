use fsdr_blocks::stream::*;
use futuresdr::blocks::{VectorSink, VectorSource};
use futuresdr::prelude::*;

#[test]
fn deinterleave_u8() -> Result<()> {
    let mut fg = Flowgraph::new();

    let deinterleaver = Deinterleave::<u8>::new();

    let orig: Vec<u8> = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink_0 = VectorSink::<u8>::new(1024);
    let vect_sink_1 = VectorSink::<u8>::new(1024);

    connect!(fg,
        src > input.deinterleaver.out0 > vect_sink_0;
        deinterleaver.out1 > vect_sink_1;
    );
    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
    let snk_0 = snk_0.items();

    let snk_1 = fg.block(&vect_sink_1)?;
    let snk_1 = snk_1.items();

    assert_eq!(snk_0.len(), orig.len() / 2);
    assert_eq!(snk_0.len(), snk_1.len());
    assert!(snk_0.iter().all(|v| *v == 0));
    assert!(snk_1.iter().all(|v| *v == 1));

    Ok(())
}
