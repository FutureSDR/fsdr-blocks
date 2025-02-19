use fsdr_blocks::channel::CrossbeamSink;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Result;
use futuresdr::runtime::Runtime;

#[test]
fn crossbeam_sink_f32() -> Result<()> {
    let mut fg = Flowgraph::new();
    let (tx, rx) = crossbeam_channel::unbounded::<Box<[f32]>>();

    let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let vector_src = VectorSource::<f32>::new(orig.clone());
    let crossbeam_sink = CrossbeamSink::<f32>::new(tx.clone());

    connect!(fg,
        vector_src > crossbeam_sink;
    );
    Runtime::new().run(fg)?;

    assert_eq!(orig, rx.recv().unwrap().to_vec());

    Ok(())
}
