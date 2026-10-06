use fsdr_blocks::cw::cw_to_char::CWToCharBuilder;
use fsdr_blocks::cw::shared::{CWAlphabet, msg_to_cw};
use futuresdr::blocks::{ChannelSource, VectorSink, VectorSource};
use futuresdr::prelude::*;

// cargo test --features="cw"
// cargo nextest run test_cw_to_char_vector --no-capture --features="cw"
#[test]
fn test_cw_to_char_vector() -> Result<()> {
    let mut fg = Flowgraph::new();

    let message = "S O__S  S".to_uppercase().chars().collect::<Vec<char>>();
    let cw = msg_to_cw(message.as_slice());
    //println!("CW-Alphabet Vector Length: {}, Content: {:?}", cw.len(), cw);

    let vector_src = VectorSource::<CWAlphabet>::new(cw);
    let cw_to_char = CWToCharBuilder::new().build();
    let vector_snk = VectorSink::<u32>::new(1024);

    connect!(fg,
        vector_src > cw_to_char > vector_snk;
    );

    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&vector_snk)?;
    let received: Vec<char> = snk
        .items()
        .iter()
        .map(|&c| char::from_u32(c).unwrap_or('_'))
        .collect();

    /*println!(
        "Char Vector Length: {}, Content: {:?}",
        received.len(),
        received
    );*/
    assert_eq!(vec!['S', ' ', 'O', '_', '_', 'S', ' ', ' ', 'S'], received);

    Ok(())
}

// cargo nextest run test_cw_to_char_channel --no-capture --features="cw"
#[test]
fn test_cw_to_char_channel() -> Result<()> {
    let mut fg = Flowgraph::new();

    let (tx, rx) = mpsc::channel::<Box<[CWAlphabet]>>(10);

    let channel_src = ChannelSource::<CWAlphabet>::new(rx);
    let cw_to_char = CWToCharBuilder::new().build();
    let vector_snk = VectorSink::<u32>::new(1024);

    connect!(fg,
        channel_src > cw_to_char > vector_snk;
    );

    let rt = Runtime::new();
    let running = rt.start(fg)?;
    let fg_term = block_on(async move {
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
        drop(tx);
        running.wait_async().await
    })?;

    let snk = fg_term.block(&vector_snk)?;
    let received: Vec<char> = snk
        .items()
        .iter()
        .map(|&c| char::from_u32(c).unwrap_or('_'))
        .collect();

    /*println!(
        "Char Vector Length: {}, Content: {:?}",
        received.len(),
        received
    );*/
    assert_eq!(vec!['S', ' ', 'O', '_', '_', 'S', 'S'], received);

    Ok(())
}
