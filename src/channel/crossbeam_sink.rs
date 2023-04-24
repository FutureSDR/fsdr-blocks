use crossbeam_channel::Sender;
use futuresdr::async_trait::async_trait;
use futuresdr::anyhow::Result;
use futuresdr::log::info;
use futuresdr::runtime::{Block, TypedBlock};
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

/// Push samples originating from a stream in a flowgraph into a crossbeam channel.
///
/// # Inputs
///
/// `in`: Samples pushed into the channel
///
/// # Usage
/// ```
/// use crossbeam_channel;
/// use fsdr_blocks::channel::CrossbeamSink;
/// use futuresdr::blocks::VectorSource;
/// use futuresdr::runtime::{Flowgraph, Runtime};
///
/// let mut fg = Flowgraph::new();
/// let (tx, rx) = crossbeam_channel::unbounded::<Box<[f32]>>();
///
/// let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
/// let vector_src = fg.add_block(VectorSource::<f32>::new(orig.clone()));
/// let crossbeam_sink = fg.add_block(CrossbeamSink::<f32>::new(tx.clone()));
///
/// fg.connect_stream(vector_src, "out", crossbeam_sink, "in").unwrap();
/// Runtime::new().run(fg).unwrap();
///
/// assert_eq!(orig, rx.recv().unwrap().to_vec());
/// ```
pub struct CrossbeamSink<T: Send + Copy + 'static> {
    sender: Sender<Box<[T]>>,
}

impl<T: Send + Copy + 'static> CrossbeamSink<T> {

    pub fn new(sender: Sender<Box<[T]>>) -> Block {
        Block::from_typed(Self::new_typed(sender))
    }

    pub fn new_typed(sender: Sender<Box<[T]>>) -> TypedBlock<Self> {
        TypedBlock::new(
            BlockMetaBuilder::new("CrossbeamSink").build(),
            StreamIoBuilder::new().add_input::<T>("in").build(),
            MessageIoBuilder::<Self>::new().build(),
            CrossbeamSink::<T> {
                sender,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<T: Send + Copy + 'static> Kernel for CrossbeamSink<T> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<T>();

        if !i.is_empty() {
                match self.sender.try_send(i.into()) {
                    Ok(_) => {
                        info!("sent data...");
                    }
                    Err(err) => {
                        info!("{}", err.to_string());
                    }
                }
            sio.input(0).consume(i.len());
        }

        if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
    }
}
