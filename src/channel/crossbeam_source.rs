use crossbeam_channel::Receiver;
use futuresdr::runtime::dev::prelude::*;

/// Pull samples from a crossbeam channel into a stream in a flowgraph.
///
/// # Outputs
///
/// `out`: Samples pulled from the channel
///
/// # Usage
/// ```
/// use crossbeam_channel;
/// use fsdr_blocks::channel::CrossbeamSource;
/// use futuresdr::prelude::*;
/// use futuresdr::blocks::VectorSink;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut fg = Flowgraph::new();
/// let (tx, rx) = crossbeam_channel::unbounded::<Box<[f32]>>();
///
/// let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
/// let src = CrossbeamSource::<f32>::new(rx.clone());
/// let snk = VectorSink::<f32>::new(1024);
///
/// tx.send(orig.clone().into()).unwrap();
/// drop(tx);
///
/// connect!(fg, src > snk);
/// Runtime::new().run(fg)?;
///
/// let snk_get = snk.get().unwrap();
/// assert_eq!(orig, *snk_get.items());
/// # Ok(())
/// # }
/// ```
#[derive(Block)]
pub struct CrossbeamSource<
    T: Send + Sync + Copy + 'static,
    O: CpuBufferWriter<Item = T> = DefaultCpuWriter<T>,
> {
    #[output]
    output: O,
    receiver: Receiver<Box<[T]>>,
    current: Option<(Box<[T]>, usize)>,
}

impl<T: Send + Sync + Copy + 'static, O: CpuBufferWriter<Item = T>> CrossbeamSource<T, O> {
    pub fn new(receiver: Receiver<Box<[T]>>) -> Self {
        CrossbeamSource {
            output: O::default(),
            receiver,
            current: None,
        }
    }
}

#[doc(hidden)]
impl<T: Send + Sync + Copy + 'static, O: CpuBufferWriter<Item = T>> Kernel
    for CrossbeamSource<T, O>
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
                    Err(crossbeam_channel::TryRecvError::Empty) => {}
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
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
