use fsdr_blocks::cw::baseband_to_cw::BaseBandToCWBuilder;
use fsdr_blocks::cw::shared::CWAlphabet::*;
use fsdr_blocks::cw::shared::{char_to_baseband, CWAlphabet};
use futuresdr::anyhow::Result;
use futuresdr::blocks::{VectorSink, VectorSinkBuilder, VectorSource};
use futuresdr::macros::connect;
use futuresdr::runtime::{Flowgraph, Runtime};

#[test]
fn test_baseband_to_cw() -> Result<()> {
    let mut fg = Flowgraph::new();

    let samples_per_dot = 1;
    let mut char_to_baseband_function = char_to_baseband(samples_per_dot);

    let message = "S O__S".to_uppercase();
    let bb = message
        .chars()
        .flat_map(|c| char_to_baseband_function(&c))
        .collect::<Vec<f32>>();
    println!("BaseBand Vector Length: {}, Content: {:?}", bb.len(), bb);

    let vector_src = VectorSource::new(bb);
    let baseband_to_cw = BaseBandToCWBuilder::new()
        .accuracy(100)
        .samples_per_dot(samples_per_dot)
        .build();
    let vector_snk = VectorSinkBuilder::<CWAlphabet>::new().build();

    connect!(fg,
        vector_src > baseband_to_cw > vector_snk;
    );

    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<CWAlphabet>>(vector_snk).unwrap();
    let received = snk.items();

    println!(
        "CW-Alphabet Vector Length: {}, Content: {:?}",
        received.len(),
        received
    );
    assert_eq!(
        &vec![
            Dot,
            Dot,
            Dot,
            WordSpace,
            Dash,
            Dash,
            Dash,
            LetterSpace,
            WordSpace,
            Dot,
            Dot,
            Dot,
        ],
        received
    );

    Ok(())
}
