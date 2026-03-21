use futuresdr::blocks::signal_source::FixedPointPhase;
use futuresdr::blocks::signal_source::NCO;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::*;

#[cfg(feature = "simd")]
use core::simd::prelude::*;

/// This blocks shift the signal in the frequency domain based on the [`NCO`] implementation.
/// Currently implemented only for float and [`Complex32`]
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
pub struct FrequencyShifter<
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    nco: NCO,
    phase_inc: FixedPointPhase,
}

impl<A, I, O> FrequencyShifter<A, I, O>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = A>,
{
    /// Create FrequencyShifter block
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let phase_inc = 2.0 * core::f32::consts::PI * frequency / sample_rate;
        let nco = NCO::new(0.0f32, phase_inc);
        Self {
            input: I::default(),
            output: O::default(),
            nco,
            phase_inc: FixedPointPhase::new(phase_inc),
        }
    }
}

pub trait FreqShiftSupported: Copy {
    fn freq_shift(nco: &mut NCO, phase_inc: FixedPointPhase, input: &[Self], output: &mut [Self]);
}

impl FreqShiftSupported for f32 {
    fn freq_shift(nco: &mut NCO, _phase_inc: FixedPointPhase, input: &[Self], output: &mut [Self]) {
        let n = input.len().min(output.len());
        for (v, r) in input[..n].iter().zip(output[..n].iter_mut()) {
            *r = (*v) * nco.phase.cos();
            nco.step();
        }
    }
}

impl FreqShiftSupported for Complex32 {
    fn freq_shift(nco: &mut NCO, phase_inc: FixedPointPhase, input: &[Self], output: &mut [Self]) {
        let n = input.len().min(output.len());
        if n == 0 {
            return;
        }

        #[cfg(feature = "simd")]
        {
            const LANES: usize = 8;
            let n_simd = n / LANES;

            if n_simd > 0 {
                let block_rotation_angle = f32::from(&phase_inc) * LANES as f32;
                let v_br_re = f32x8::splat(block_rotation_angle.cos());
                let v_br_im = f32x8::splat(block_rotation_angle.sin());

                let mut temp_nco = nco.clone();
                let mut cos_arr = [0.0f32; LANES];
                let mut sin_arr = [0.0f32; LANES];
                for j in 0..LANES {
                    cos_arr[j] = temp_nco.phase.cos();
                    sin_arr[j] = temp_nco.phase.sin();
                    temp_nco.step();
                }
                let mut block_phasor_cos = f32x8::from_array(cos_arr);
                let mut block_phasor_sin = f32x8::from_array(sin_arr);

                let i_f32 =
                    unsafe { core::slice::from_raw_parts(input.as_ptr() as *const f32, n * 2) };
                let o_f32 = unsafe {
                    core::slice::from_raw_parts_mut(output.as_mut_ptr() as *mut f32, n * 2)
                };

                for i in 0..n_simd {
                    let v0 = f32x8::from_slice(&i_f32[i * LANES * 2..i * LANES * 2 + 8]);
                    let v1 = f32x8::from_slice(&i_f32[i * LANES * 2 + 8..i * LANES * 2 + 16]);
                    let (v_re, v_im) = v0.deinterleave(v1);

                    let res_re = v_re * block_phasor_cos - v_im * block_phasor_sin;
                    let res_im = v_re * block_phasor_sin + v_im * block_phasor_cos;

                    let (o0, o1) = res_re.interleave(res_im);
                    o0.copy_to_slice(&mut o_f32[i * LANES * 2..i * LANES * 2 + 8]);
                    o1.copy_to_slice(&mut o_f32[i * LANES * 2 + 8..i * LANES * 2 + 16]);

                    let next_cos = block_phasor_cos * v_br_re - block_phasor_sin * v_br_im;
                    let next_sin = block_phasor_cos * v_br_im + block_phasor_sin * v_br_re;
                    block_phasor_cos = next_cos;
                    block_phasor_sin = next_sin;
                }
                nco.steps((n_simd * LANES) as i32);
            }

            let tail_start = n_simd * LANES;
            if tail_start < n {
                let rotation = Complex32::new(phase_inc.cos(), phase_inc.sin());
                let mut current_phasor = Complex32::new(nco.phase.cos(), nco.phase.sin());
                for (v, r) in input[tail_start..n]
                    .iter()
                    .zip(output[tail_start..n].iter_mut())
                {
                    *r = (*v) * current_phasor;
                    current_phasor *= rotation;
                }
                nco.steps((n - tail_start) as i32);
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            let rotation = Complex32::new(phase_inc.cos(), phase_inc.sin());
            let mut current_phasor = Complex32::new(nco.phase.cos(), nco.phase.sin());
            for (v, r) in input[..n].iter().zip(output[..n].iter_mut()) {
                *r = (*v) * current_phasor;
                current_phasor *= rotation;
            }
            nco.steps(n as i32);
        }
    }
}

#[doc(hidden)]
impl<A, I, O> Kernel for FrequencyShifter<A, I, O>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy + FreqShiftSupported,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = A>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let m = {
            let i = self.input.slice();
            let o = self.output.slice();

            let m = std::cmp::min(i.len(), o.len());
            if m > 0 {
                A::freq_shift(&mut self.nco, self.phase_inc, &i[..m], &mut o[..m]);
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
