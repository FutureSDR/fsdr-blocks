use fsdr_blocks::async_channel::AsyncChannelSource;
use futuresdr::blocks::{Head, VectorSink, VectorSinkBuilder};
use futuresdr::macros::connect;
use futuresdr::runtime::Result;
use futuresdr::runtime::{Flowgraph, Runtime};

#[test]
fn run_async_channel_source_u32() -> Result<()> {
    tokio_test::block_on(async_channel_source_u32())
}

async fn async_channel_source_u32() -> Result<()> {
    let mut fg = Flowgraph::new();
    let orig = vec![0, 1, 2];
    let (tx, rx) = async_channel::unbounded::<Box<[u32]>>();

    let async_channel_src = AsyncChannelSource::<u32>::new(rx);
    let limit = Head::<u32>::new(orig.len() as u64);
    let vector_snk = VectorSinkBuilder::<u32>::new().build();

    connect!(fg,
        async_channel_src > limit > vector_snk;
    );

    tx.send(orig.clone().into_boxed_slice()).await.unwrap();
    tx.close();

    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<u32>>(vector_snk).unwrap();
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
