use async_channel::Sender;
use futuresdr::runtime::dev::prelude::*;

/// Push samples originating from a stream in a flowgraph into an async channel.
///
/// # Inputs
///
/// `in`: Samples pushed into the channel
///
/// # Usage
/// ```
/// use async_channel;
/// use fsdr_blocks::async_channel::AsyncChannelSink;
/// use futuresdr::prelude::*;
/// use futuresdr::blocks::VectorSource;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut fg = Flowgraph::new();
/// let (tx, rx) = async_channel::unbounded::<Box<[f32]>>();
///
/// let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
/// let src = VectorSource::<f32>::new(orig.clone());
/// let snk = AsyncChannelSink::<f32>::new(tx.clone());
///
/// connect!(fg, src > snk);
/// Runtime::new().run(fg)?;
///
/// assert_eq!(orig, futuresdr::async_io::block_on(rx.recv()).unwrap().to_vec());
/// # Ok(())
/// # }
/// ```
#[derive(Block)]
pub struct AsyncChannelSink<
    T: Send + Sync + Clone + 'static,
    I: CpuBufferReader<Item = T> = DefaultCpuReader<T>,
> {
    #[input]
    input: I,
    sender: Sender<Box<[T]>>,
}

impl<T: Send + Sync + Clone + 'static, I: CpuBufferReader<Item = T>> AsyncChannelSink<T, I> {
    pub fn new(sender: Sender<Box<[T]>>) -> Self {
        AsyncChannelSink {
            input: I::default(),
            sender,
        }
    }
}

#[doc(hidden)]
impl<T: Send + Sync + Clone + 'static, I: CpuBufferReader<Item = T>> Kernel
    for AsyncChannelSink<T, I>
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();

        let i_len = i.len();
        if i_len > 0 {
            match self.sender.try_send(i.into()) {
                Ok(_) => {
                    self.input.consume(i_len);
                }
                Err(async_channel::TrySendError::Full(_)) => {}
                Err(async_channel::TrySendError::Closed(_)) => {
                    io.finished = true;
                }
            }
        }

        if self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
