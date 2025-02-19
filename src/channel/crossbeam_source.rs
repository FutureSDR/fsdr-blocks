use async_trait::async_trait;
use crossbeam_channel::{Receiver, TryRecvError};
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::Result;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::runtime::{Block, TypedBlock};

/// Push samples from a channel into a flowgraph stream connection.
///
/// # Outputs
///
/// `out`: Samples pushed into the channel
///
/// # Usage
/// ```
/// use crossbeam_channel;
/// use fsdr_blocks::channel::CrossbeamSource;
/// use futuresdr::blocks::{Head, VectorSink, VectorSinkBuilder};
/// use futuresdr::log::debug;
/// use futuresdr::runtime::{Flowgraph, Runtime};
///
/// let mut fg = Flowgraph::new();
/// let orig = vec![0, 1, 2];
/// let (tx, rx) = crossbeam_channel::unbounded::<Box<[u32]>>();
///
/// let crossbeam_source = fg.add_block(CrossbeamSource::<u32>::new(rx));
/// let limit = fg.add_block(Head::<u32>::new(orig.len() as u64));
/// let vector_sink = fg.add_block(VectorSinkBuilder::<u32>::new().build());
///
/// fg.connect_stream(crossbeam_source, "out", limit, "in").unwrap();
/// fg.connect_stream(limit, "out", vector_sink, "in").unwrap();
/// tx.try_send(orig.clone().into_boxed_slice()).unwrap();
///
/// fg = Runtime::new().run(fg).unwrap();
///
/// let snk = fg.kernel::<VectorSink<u32>>(vector_sink).unwrap();
/// let received = snk.items();
///
/// debug!("{}", received.len());
/// debug!("{}", orig.len());
///
/// assert_eq!(received.len(), orig.len());
///
/// for (v, e) in orig.iter().zip(received.iter()) {
///     debug!("{v} == {e}");
///     assert_eq!(v, e);
/// }
/// ```
pub struct CrossbeamSource<T: Send + 'static> {
    receiver: Receiver<Box<[T]>>,
    current: Option<(Box<[T]>, usize)>,
}

impl<T: Send + 'static> CrossbeamSource<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(receiver: Receiver<Box<[T]>>) -> Block {
        Block::from_typed(Self::new_typed(receiver))
    }

    pub fn new_typed(receiver: Receiver<Box<[T]>>) -> TypedBlock<Self> {
        TypedBlock::new(
            BlockMetaBuilder::new("CrossbeamSource").build(),
            StreamIoBuilder::new().add_output::<T>("out").build(),
            MessageIoBuilder::new().build(),
            CrossbeamSource::<T> {
                receiver,
                current: None,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<T: Send + 'static> Kernel for CrossbeamSource<T> {
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
            match self.receiver.try_recv() {
                //.by_ref().next().await {
                Ok(data) => {
                    // debug!("received data chunk on channel");
                    self.current = Some((data, 0));
                }
                Err(TryRecvError::Empty) => {
                    //debug!("channel empty");
                }
                Err(TryRecvError::Disconnected) => {
                    // debug!("sender-end of channel was closed");
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
