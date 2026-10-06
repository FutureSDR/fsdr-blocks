use futuresdr::runtime::dev::prelude::*;

/// This blocks deinterleave a unique stream into two separate stream.
/// Typically used to deinterleave iq stream into of stream for `i` and one for `q`.
///
/// # Usage
/// ```
/// use fsdr_blocks::stream::Deinterleave;
/// let blk = Deinterleave::<f32>::new();
/// ```
#[derive(Block)]
pub struct Deinterleave<
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O0: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
    O1: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    #[input]
    input: I,
    #[output]
    out0: O0,
    #[output]
    out1: O1,
    first: bool,
}

impl<A, I, O0, O1> Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    pub fn new() -> Self {
        Self {
            input: I::default(),
            out0: O0::default(),
            out1: O1::default(),
            first: true,
        }
    }
}

impl<A, I, O0, O1> Default for Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<A, I, O0, O1> Kernel for Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (m, m0, m1) = {
            let i0 = self.input.slice();
            let o0 = self.out0.slice();
            let o1 = self.out1.slice();

            let mut m0 = 0;
            let mut m1 = 0;

            if !i0.is_empty() {
                let mut i_idx = 0;

                if !self.first && i_idx < i0.len() && m1 < o1.len() {
                    o1[m1] = i0[i_idx];
                    m1 += 1;
                    i_idx += 1;
                    self.first = true;
                }

                let pairs = std::cmp::min(
                    (i0.len() - i_idx) / 2,
                    std::cmp::min(o0.len() - m0, o1.len() - m1),
                );
                if pairs > 0 {
                    let in_chunks = &i0[i_idx..i_idx + pairs * 2];
                    let out0_slice = &mut o0[m0..m0 + pairs];
                    let out1_slice = &mut o1[m1..m1 + pairs];

                    #[allow(clippy::chunks_exact_to_as_chunks)]
                    for (chunk, (d0, d1)) in in_chunks
                        .chunks_exact(2)
                        .zip(out0_slice.iter_mut().zip(out1_slice.iter_mut()))
                    {
                        *d0 = chunk[0];
                        *d1 = chunk[1];
                    }
                    m0 += pairs;
                    m1 += pairs;
                    i_idx += pairs * 2;
                }

                if i_idx < i0.len() && self.first && m0 < o0.len() {
                    o0[m0] = i0[i_idx];
                    m0 += 1;
                    self.first = false;
                }
            }

            (m0 + m1, m0, m1)
        };

        self.input.consume(m);
        self.out0.produce(m0);
        self.out1.produce(m1);

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}
