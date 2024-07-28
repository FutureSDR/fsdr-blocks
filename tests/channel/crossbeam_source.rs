use fsdr_blocks::channel::CrossbeamSource;
use futuresdr::anyhow::Result;
use futuresdr::blocks::{Head, VectorSink, VectorSinkBuilder};
// use futuresdr::log::debug;
use futuresdr::macros::connect;
use futuresdr::runtime::{Flowgraph, Runtime};

#[test]
fn crossbeam_source_u32() -> Result<()> {
    let mut fg = Flowgraph::new();
    let orig = vec![0, 1, 2];
    let (tx, rx) = crossbeam_channel::unbounded::<Box<[u32]>>();

    let crossbeam_source = CrossbeamSource::<u32>::new(rx);
    let limit = Head::<u32>::new(orig.len() as u64);
    let vector_sink = VectorSinkBuilder::<u32>::new().build();

    connect!(fg,
        crossbeam_source > limit > vector_sink;
    );

    tx.try_send(orig.clone().into_boxed_slice()).unwrap();

    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<u32>>(vector_sink).unwrap();
    let received = snk.items();

    // debug!("{}", received.len());
    // debug!("{}", orig.len());

    assert_eq!(received.len(), orig.len());

    for (v, e) in orig.iter().zip(received.iter()) {
        // debug!("{v} == {e}");
        assert_eq!(v, e);
    }
    Ok(())
}
