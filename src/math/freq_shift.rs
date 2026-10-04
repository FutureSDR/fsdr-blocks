use futuresdr::blocks::signal_source::FixedPointPhase;
use futuresdr::blocks::signal_source::NCO;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;

/// This block shifts the signal in the frequency domain based on the [`NCO`] implementation.
/// Implemented for float (`f32`) and [`Complex32`].
///
/// # Usage
///
/// ```
/// # use futuresdr::num_complex::Complex32;
/// # use fsdr_blocks::math::FrequencyShifter;
/// # let freq = 2_000;
/// # let sample_rate = 48_000;
/// let blk = FrequencyShifter::<Complex32>::new(freq as f32, sample_rate as f32);
/// ```
#[derive(Block)]
#[message_inputs(set_frequency)]
pub struct FrequencyShifter<
    A: Send + Sync + Default + Clone + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    freq: f32,
    sample_rate: f32,
    #[input]
    input: I,
    #[output]
    output: O,
    nco: NCO,
    phase_inc: FixedPointPhase,
}

impl<A, I, O> FrequencyShifter<A, I, O>
where
    A: Send + Sync + Default + Clone + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = A>,
{
    /// Create FrequencyShifter block
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let phase_inc = 2.0 * core::f32::consts::PI * frequency / sample_rate;
        let nco = NCO::new(0.0f32, phase_inc);
        Self {
            freq: frequency,
            sample_rate,
            input: I::default(),
            output: O::default(),
            nco,
            phase_inc: FixedPointPhase::new(phase_inc),
        }
    }

    async fn set_frequency(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        let freq = match p {
            Pmt::F32(f) => f,
            Pmt::F64(f) => f as f32,
            Pmt::U32(f) => f as f32,
            Pmt::U64(f) => f as f32,
            _ => return Ok(Pmt::InvalidValue),
        };
        self.freq = freq;
        let rad_inc = 2.0 * core::f32::consts::PI * freq / self.sample_rate;
        self.phase_inc = FixedPointPhase::new(rad_inc);
        Ok(Pmt::Ok)
    }
}

#[inline(always)]
fn fast_complex_mul(a: Complex32, b: Complex32) -> Complex32 {
    unsafe {
        let re_re = core::intrinsics::fmul_fast(a.re, b.re);
        let im_im = core::intrinsics::fmul_fast(a.im, b.im);
        let re_im = core::intrinsics::fmul_fast(a.re, b.im);
        let im_re = core::intrinsics::fmul_fast(a.im, b.re);

        let re = core::intrinsics::fsub_fast(re_re, im_im);
        let im = core::intrinsics::fadd_fast(re_im, im_re);

        Complex32::new(re, im)
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
        _meta: &BlockMeta,
    ) -> Result<()> {
        let m = {
            let i = self.input.slice();
            let o = self.output.slice();

            let m = std::cmp::min(i.len(), o.len());
            if m > 0 {
                for (v, r) in i[..m].iter().zip(o[..m].iter_mut()) {
                    let cos_val = self.nco.phase.cos();
                    *r = unsafe { core::intrinsics::fmul_fast(*v, cos_val) };
                    self.nco.step();
                }
            }
            m
        };

        if m > 0 {
            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && self.input.slice().is_empty() {
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
        _meta: &BlockMeta,
    ) -> Result<()> {
        let m = {
            let i = self.input.slice();
            let o = self.output.slice();

            let m = std::cmp::min(i.len(), o.len());
            if m > 0 {
                let rotation = Complex32::new(self.phase_inc.cos(), self.phase_inc.sin());
                let mut current_phasor = Complex32::new(self.nco.phase.cos(), self.nco.phase.sin());
                let mut count = 0usize;

                for (v, r) in i[..m].iter().zip(o[..m].iter_mut()) {
                    *r = fast_complex_mul(*v, current_phasor);
                    current_phasor = fast_complex_mul(current_phasor, rotation);
                    count += 1;
                    if count & 0xFF == 0 {
                        let norm_sq = current_phasor.re * current_phasor.re
                            + current_phasor.im * current_phasor.im;
                        if (norm_sq - 1.0).abs() > 1e-4 {
                            let inv_norm = 1.0 / norm_sq.sqrt();
                            current_phasor.re =
                                unsafe { core::intrinsics::fmul_fast(current_phasor.re, inv_norm) };
                            current_phasor.im =
                                unsafe { core::intrinsics::fmul_fast(current_phasor.im, inv_norm) };
                        }
                    }
                }
                self.nco.steps(m as i32);
            }
            m
        };

        if m > 0 {
            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}
