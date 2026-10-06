use async_channel::Receiver;
use futuresdr::runtime::dev::prelude::*;

/// Push samples through a channel into a stream connection.
///
/// # Outputs
///
/// `out`: Samples pushed into the channel
///
/// # Usage
/// ```
/// use async_channel;
/// use fsdr_blocks::async_channel::AsyncChannelSource;
/// use futuresdr::runtime::Flowgraph;
///
/// let mut fg = Flowgraph::new();
/// let (tx, rx) = async_channel::unbounded::<Box<[u32]>>();
///
/// let async_channel_src = fg.add_block(AsyncChannelSource::<u32>::new(rx));
///
/// // tx.send(orig.clone().into_boxed_slice()).await.unwrap();
/// ```
#[derive(Block)]
pub struct AsyncChannelSource<
    T: Send + Sync + Default + Clone + Copy + std::fmt::Debug + 'static,
    O: CpuBufferWriter<Item = T> = DefaultCpuWriter<T>,
> {
    #[output]
    output: O,
    receiver: Receiver<Box<[T]>>,
    current: Option<(Box<[T]>, usize)>,
}

impl<
    T: Send + Sync + Default + Clone + Copy + std::fmt::Debug + 'static,
    O: CpuBufferWriter<Item = T>,
> AsyncChannelSource<T, O>
{
    pub fn new(receiver: Receiver<Box<[T]>>) -> Self {
        AsyncChannelSource {
            output: O::default(),
            receiver,
            current: None,
        }
    }
}

#[doc(hidden)]
impl<T, O> Kernel for AsyncChannelSource<T, O>
where
    T: Send + Sync + Default + Clone + Copy + std::fmt::Debug + 'static,
    O: CpuBufferWriter<Item = T>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (produced, call_again, finished) = {
            let out = self.output.slice();
            if out.is_empty() {
                return Ok(());
            }

            let mut finished = false;
            if self.current.is_none() {
                match self.receiver.try_recv() {
                    Ok(data) => {
                        self.current = Some((data, 0));
                    }
                    Err(async_channel::TryRecvError::Empty) => {}
                    Err(async_channel::TryRecvError::Closed) => {
                        finished = true;
                    }
                }
            }

            let mut produced = 0;
            if let Some((data, index)) = &mut self.current {
                let n = std::cmp::min(data.len() - *index, out.len());
                out[..n].copy_from_slice(&data[*index..*index + n]);
                produced = n;
                *index += n;
                if *index == data.len() {
                    self.current = None;
                }
            }
            (produced, self.current.is_none(), finished)
        };

        if produced > 0 {
            self.output.produce(produced);
        }
        io.call_again = call_again;
        if finished {
            io.finished = true;
        }

        Ok(())
    }
}
