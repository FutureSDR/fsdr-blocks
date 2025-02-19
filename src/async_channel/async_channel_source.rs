use async_channel::Receiver;
use futuresdr::futures::StreamExt;

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

/// Push samples through a channel into a stream connection.
///
/// # Outputs
///
/// `out`: Samples pushed into the channel
///
/// # Usage
/// ```
/// use async_channel;
/// use fsdr-blocks::blocks::AsyncChannelSource;
/// use futuresdr::runtime::Flowgraph;
///
/// let mut fg = Flowgraph::new();
/// let (tx, rx) = async_channel::unbounded::<Box<[u32]>>();
///
/// let async_channel_src = fg.add_block(AsyncChannelSource::<u32>::new(rx));
///
/// tx.send(orig.clone().into_boxed_slice()).await.unwrap();
/// ```
pub struct AsyncChannelSource<T: Send + 'static> {
    receiver: Receiver<Box<[T]>>,
    current: Option<(Box<[T]>, usize)>,
}

impl<T: Send + 'static> AsyncChannelSource<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(receiver: Receiver<Box<[T]>>) -> Block {
        Block::new(
            BlockMetaBuilder::new("AsyncChannelSource").build(),
            StreamIoBuilder::new().add_output::<T>("out").build(),
            MessageIoBuilder::new().build(),
            AsyncChannelSource::<T> {
                receiver,
                current: None,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<T: Send + 'static> Kernel for AsyncChannelSource<T> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let out = sio.output(0).slice::<T>();
        if out.is_empty() {
            return Ok(());
        }

        if self.current.is_none() {
            match self.receiver.by_ref().recv().await {
                //.by_ref().next().await
                Ok(data) => {
                    // info!("received data chunk on channel");
                    self.current = Some((data, 0));
                }
                Err(_err) => {
                    // info!("sender-end of channel was closed");
                    io.finished = true;
                    return Ok(());
                }
            }
        }

        if let Some((data, index)) = &mut self.current {
            let n = std::cmp::min(data.len() - *index, out.len());
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr().add(*index), out.as_mut_ptr(), n);
            };
            sio.output(0).produce(n);
            *index += n;
            if *index == data.len() {
                self.current = None;
            }
        }

        if self.current.is_none() {
            io.call_again = true;
        }

        Ok(())
    }
}
