use fsdr_blocks::cw::cw_to_char::CWToCharBuilder;
use fsdr_blocks::cw::shared::{msg_to_cw, CWAlphabet};
use futuresdr::anyhow::Result;
use futuresdr::async_io::block_on;
use futuresdr::blocks::{ChannelSource, VectorSink, VectorSinkBuilder, VectorSource};
use futuresdr::futures::SinkExt;
use futuresdr::macros::connect;
use futuresdr::runtime::{Flowgraph, Runtime};

// cargo test --features="cw"
// cargo nextest run test_cw_to_char_vector --no-capture --features="cw"
#[test]
fn test_cw_to_char_vector() -> Result<()> {
    let mut fg = Flowgraph::new();

    let message = "S O__S  S".to_uppercase().chars().collect::<Vec<char>>();
    let cw = msg_to_cw(message.as_slice());
    //println!("CW-Alphabet Vector Length: {}, Content: {:?}", cw.len(), cw);

    let vector_src = VectorSource::new(cw);
    let cw_to_char = CWToCharBuilder::new().build();
    let vector_snk = VectorSinkBuilder::<char>::new().build();

    connect!(fg,
        vector_src > cw_to_char;
        cw_to_char > vector_snk;
    );

    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<char>>(vector_snk).unwrap();
    let received = snk.items();

    /*println!(
        "Char Vector Length: {}, Content: {:?}",
        received.len(),
        received
    );*/
    assert_eq!(&vec!['S', ' ', 'O', '_', ' ', ' ', 'S'], received);

    Ok(())
}

// cargo nextest run test_cw_to_char_channel --no-capture --features="cw"
#[test]
fn test_cw_to_char_channel() -> Result<()> {
    let mut fg = Flowgraph::new();

    let (mut tx, rx) = futuresdr::futures::channel::mpsc::channel::<Box<[CWAlphabet]>>(10);

    let channel_src = ChannelSource::<CWAlphabet>::new(rx);
    let cw_to_char = CWToCharBuilder::new().build();
    let vector_snk = VectorSinkBuilder::<char>::new().build();

    connect!(fg,
        channel_src > cw_to_char > vector_snk;
    );

    let rt = Runtime::new();
    let fg = block_on(async move {
        let (fg, _) = rt.start(fg).await;
        let c = msg_to_cw(['S'].as_slice()).into_boxed_slice();
        tx.send(c).await.unwrap();
        let c = msg_to_cw([' '].as_slice()).into_boxed_slice();
        tx.send(c).await.unwrap();
        let c = msg_to_cw(['O'].as_slice()).into_boxed_slice();
        tx.send(c).await.unwrap();
        let c = msg_to_cw(['_'].as_slice()).into_boxed_slice();
        tx.send(c).await.unwrap();
        let c = msg_to_cw(['_', 'S'].as_slice()).into_boxed_slice();
        tx.send(c).await.unwrap();
        let c = msg_to_cw(['S'].as_slice()).into_boxed_slice();
        tx.send(c).await.unwrap();
        tx.close().await.unwrap();
        fg.await // as Result<Flowgraph>
    })?;

    let snk = fg.kernel::<VectorSink<char>>(vector_snk).unwrap();
    let received = snk.items();

    /*println!(
        "Char Vector Length: {}, Content: {:?}",
        received.len(),
        received
    );*/
    assert_eq!(&vec!['S', ' ', 'O', '_', 'S'], received);

    Ok(())
}
