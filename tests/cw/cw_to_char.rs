use futuresdr::anyhow::Result;
use futuresdr::blocks::{VectorSink, VectorSinkBuilder, VectorSource};
use futuresdr::macros::connect;
use futuresdr::runtime::{Flowgraph, Runtime};
use fsdr_blocks::cw::cw_to_char::CWToCharBuilder;
use fsdr_blocks::cw::shared::msg_to_cw;

#[test]
fn test_cw_to_char() -> Result<()> {
    let mut fg = Flowgraph::new();

    let message = "S O__S".to_uppercase().chars().collect::<Vec<char>>();
    let cw = msg_to_cw(message.as_slice());
    println!("CW-Alphabet Vector Length: {}, Content: {:?}", cw.len(), cw);

    let vector_src = VectorSource::new(cw);
    let cw_to_char = CWToCharBuilder::new().build();
    let vector_snk = VectorSinkBuilder::<char>::new().build();

    connect!(fg,
        vector_src > cw_to_char > vector_snk;
    );

    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<char>>(vector_snk).unwrap();
    let received = snk.items();

    println!("Char Vector Length: {}, Content: {:?}", received.len(), received);
    //assert_eq!(&vec!['S', 'O', 'S'], received);

    Ok(())
}