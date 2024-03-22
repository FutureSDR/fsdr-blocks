use fsdr_blocks::async_channel::AsyncChannelSink;
use futuresdr::anyhow::Result;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[test]
fn run_async_channel_sink_f32() -> Result<()> {
    tokio_test::block_on(async_channel_sink_f32())
}

async fn async_channel_sink_f32() -> Result<()> {
    let mut fg = Flowgraph::new();
    let (tx, rx) = async_channel::unbounded::<Box<[f32]>>();

    let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let vector_src = VectorSource::<f32>::new(orig.clone());
    let async_channel_snk = AsyncChannelSink::<f32>::new(tx.clone());

    connect!(fg,
        vector_src > async_channel_snk;
    );
    Runtime::new().run(fg)?;

    assert_eq!(orig, rx.recv().await.unwrap().to_vec());

    Ok(())
}
