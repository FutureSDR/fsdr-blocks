use futuresdr::blocks::signal_source::NCO;
use futuresdr::macros::Block;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::{
    BlockMeta, CpuBufferReader, CpuBufferWriter, DefaultCpuReader, DefaultCpuWriter, Kernel,
    MessageOutputs, WorkIo, Result,
};

/// This block shifts the signal in the frequency domain based on the [`NCO`] implementation.
/// Currently implemented only for float and [`Complex32`]
///
/// # Usage
///
/// ```
/// use futuresdr::num_complex::Complex32;
/// use fsdr_blocks::math::FrequencyShifter;
/// let freq = 2_000;
/// let sample_rate = 48_000;
/// let blk = FrequencyShifter::<Complex32>::new(freq as f32, sample_rate as f32);
/// ```
#[derive(Block)]
pub struct FrequencyShifter<T, I = DefaultCpuReader<T>, O = DefaultCpuWriter<T>>
where
    T: Copy + Send + 'static,
    I: CpuBufferReader<Item = T>,
    O: CpuBufferWriter<Item = T>,
{
    #[input]
    input: I,
    #[output]
    output: O,
    nco: NCO,
}

impl<T, I, O> FrequencyShifter<T, I, O>
where
    T: Copy + Send + 'static,
    I: CpuBufferReader<Item = T>,
    O: CpuBufferWriter<Item = T>,
{
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            input: I::default(),
            output: O::default(),
            nco: NCO::new(0.0f32, core::f32::consts::TAU * frequency / sample_rate),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FrequencyShifter<f32, I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();
        let i_len = i.len();

        let m = std::cmp::min(i_len, o.len());
        if m > 0 {
            for (v, r) in i.iter().zip(o.iter_mut()) {
                *r = (*v) * self.nco.phase.cos();
                self.nco.step();
            }

            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && m == i_len {
            io.finished = true;
        }

        Ok(())
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FrequencyShifter<Complex32, I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();
        let i_len = i.len();

        let m = std::cmp::min(i_len, o.len());
        if m > 0 {
            for (v, r) in i.iter().zip(o.iter_mut()) {
                *r = (*v) * Complex32::new(self.nco.phase.cos(), self.nco.phase.sin());
                self.nco.step();
            }

            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && m == i_len {
            io.finished = true;
        }

        Ok(())
    }
}
