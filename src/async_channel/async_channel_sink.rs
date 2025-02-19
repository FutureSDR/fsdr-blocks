use async_channel::Sender;

use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::Result;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

/// Get samples out of a Flowgraph into a channel.
///
/// # Inputs
///
/// `in`: Samples retrieved from teh flowgraph
///
/// # Usage
/// ```
/// use async_channel;
/// use futuresdr::blocks::VectorSource;
/// use fsdr-blocks::blocks::AsyncChannelSink
/// use futuresdr::runtime::Flowgraph;
///
/// let mut fg = Flowgraph::new();
/// let (tx, rx) = async_channel::unbounded::<Box<[u32]>>();
/// let vec = vec![0, 1, 2];
/// let src = fg.add_block(VectorSource::<u32>::new(vec));
/// let cs = fg.add_block(AsyncChannelSink::<u32>::new(tx));
/// // start flowgraph
/// ```
pub struct AsyncChannelSink<T: Send + 'static> {
    sender: Sender<Box<[T]>>,
}

impl<T: Send + Clone + 'static> AsyncChannelSink<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(sender: Sender<Box<[T]>>) -> Block {
        Block::new(
            BlockMetaBuilder::new("AsyncChannelSink").build(),
            StreamIoBuilder::new().add_input::<T>("in").build(),
            MessageIoBuilder::new().build(),
            AsyncChannelSink::<T> { sender },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<T: Send + Clone + 'static> Kernel for AsyncChannelSink<T> {
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
                    // info!("sent data...");
                }
                Err(_err) => {
                    // info!("{}", err.to_string());
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
