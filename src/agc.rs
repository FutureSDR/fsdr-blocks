use futuresdr::futuredsp::num_traits::ToPrimitive;
use futuresdr::num_complex::ComplexFloat;
use futuresdr::runtime::dev::prelude::*;

/// Automatic Gain Control Block
#[derive(Block)]
#[message_inputs(auto_lock, gain_lock, max_gain, adjustment_rate, reference_power)]
pub struct Agc<
    T: Send + Sync + ComplexFloat<Real: ToPrimitive> + Default + std::fmt::Debug + Copy + 'static,
    I: CpuBufferReader<Item = T> = DefaultCpuReader<T>,
    O: CpuBufferWriter<Item = T> = DefaultCpuWriter<T>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    /// Minimum value that has to be reached in order for AGC to start adjusting gain.
    squelch: f32,
    /// maximum gain value
    max_gain: f32,
    /// initial gain value.
    gain: f32,
    /// reference value to adjust signal power to.
    reference_power: f32,
    /// the update rate of the loop.
    adjustment_rate: f32,
    /// Set when gain should not be adjusted anymore, but rather be locked to the current value
    gain_lock: bool,
    /// Set when gain should be automatically locked, when reference power is reached.
    auto_lock: bool,
}

impl<T, I, O> Agc<T, I, O>
where
    T: Send + Sync + ComplexFloat<Real: ToPrimitive> + Default + std::fmt::Debug + Copy + 'static,
    I: CpuBufferReader<Item = T>,
    O: CpuBufferWriter<Item = T>,
{
    /// Create AGC Block
    pub fn new(
        squelch: f32,
        max_gain: f32,
        gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        gain_lock: bool,
        auto_lock: bool,
    ) -> Self {
        Self {
            input: I::default(),
            output: O::default(),
            squelch,
            max_gain,
            gain,
            reference_power,
            adjustment_rate,
            gain_lock,
            auto_lock,
        }
    }

    async fn auto_lock(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::Bool(l) = p {
            self.auto_lock = l;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn gain_lock(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::Bool(l) = p {
            self.gain_lock = l;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn max_gain(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(r) = p {
            self.max_gain = r;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn adjustment_rate(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(r) = p {
            self.adjustment_rate = r;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn reference_power(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(r) = p {
            self.reference_power = r;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }
}

#[doc(hidden)]
impl<T, I, O> Kernel for Agc<T, I, O>
where
    T: Send + Sync + ComplexFloat<Real: ToPrimitive> + Default + std::fmt::Debug + Copy + 'static,
    I: CpuBufferReader<Item = T>,
    O: CpuBufferWriter<Item = T>,
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
                let squelch = self.squelch;
                let max_gain = self.max_gain;
                let mut gain = self.gain;
                let mut gain_lock = self.gain_lock;
                let auto_lock = self.auto_lock;
                let reference_power = self.reference_power;
                let adjustment_rate = self.adjustment_rate;

                let dynamic_adjustment_rate = if adjustment_rate > 0.0 {
                    adjustment_rate
                } else {
                    0.0001
                };

                for (src, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                    let re_in = src.re().to_f32().unwrap_or(0.0);
                    let im_in = src.im().to_f32().unwrap_or(0.0);
                    let input_power = re_in.mul_add(re_in, im_in.algebraic_mul(im_in));

                    if input_power > squelch {
                        let output = (*src) * T::from(gain).unwrap();
                        let re_out = output.re().to_f32().unwrap_or(0.0);
                        let im_out = output.im().to_f32().unwrap_or(0.0);
                        let output_power = re_out.mul_add(re_out, im_out.algebraic_mul(im_out));

                        if auto_lock {
                            if input_power > reference_power {
                                if output_power < reference_power {
                                    gain_lock = true;
                                }
                            } else if output_power > reference_power {
                                gain_lock = true;
                            }
                        }

                        if !gain_lock {
                            let err = f32::algebraic_sub(reference_power, output_power);
                            gain = err
                                .mul_add(dynamic_adjustment_rate, gain)
                                .clamp(0.0, max_gain);
                        }
                        *dst = output;
                    } else {
                        *dst = T::from(0.0).unwrap();
                    }
                }
                self.gain = gain;
                self.gain_lock = gain_lock;
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

/// Builder for [`Agc`] block
pub struct AgcBuilder<T> {
    squelch: f32,
    /// maximum gain value (0 for unlimited).
    max_gain: f32,
    /// initial gain value.
    gain: f32,
    /// reference value to adjust signal power to.
    reference_power: f32,
    /// the update rate of the loop.
    adjustment_rate: f32,
    /// Set when gain should not be adjusted anymore, but rather be locked to the current value
    gain_lock: bool,
    /// Set when gain should be automatically locked, when reference power is reached.
    auto_lock: bool,
    _type: std::marker::PhantomData<T>,
}

impl<T> AgcBuilder<T>
where
    T: Send + Sync + ComplexFloat<Real: ToPrimitive> + Default + std::fmt::Debug + Copy + 'static,
{
    /// Create builder w/ default parameters
    ///
    /// ## Defaults
    /// - `squelch`: 0.0
    /// - `max_gain`: 65536.0
    /// - `gain`: 1.0
    /// - `reference_power`: 1.0
    /// - `adjustment_rate`: 0.0001
    /// - `gain_lock`: false
    /// - `auto_lock`: false
    pub fn new() -> AgcBuilder<T> {
        AgcBuilder {
            squelch: 0.0,
            max_gain: 65536.0,
            gain: 1.0,
            reference_power: 1.0,
            adjustment_rate: 0.0001,
            gain_lock: false,
            auto_lock: false,
            _type: std::marker::PhantomData,
        }
    }

    /// Surpress signals below this level
    pub fn squelch(mut self, squelch: f32) -> AgcBuilder<T> {
        self.squelch = squelch;
        self
    }

    /// Max gain to use to bring input closer to reference level
    pub fn max_gain(mut self, max_gain: f32) -> AgcBuilder<T> {
        self.max_gain = max_gain;
        self
    }

    /// Adjustment rate, i.e., impact of current sample on gain setting
    pub fn adjustment_rate(mut self, adjustment_rate: f32) -> AgcBuilder<T> {
        self.adjustment_rate = adjustment_rate;
        self
    }

    /// Set initial gain value
    pub fn gain(mut self, gain: f32) -> AgcBuilder<T> {
        self.gain = gain;
        self
    }

    /// Set initial gain value (alias for gain)
    pub fn initial_gain(self, gain: f32) -> AgcBuilder<T> {
        self.gain(gain)
    }

    /// Targeted power level
    pub fn reference_power(mut self, reference_power: f32) -> AgcBuilder<T> {
        self.reference_power = reference_power;
        self
    }

    /// Fix gain setting, disabling AGC
    pub fn gain_lock(mut self, gain_lock: bool) -> AgcBuilder<T> {
        self.gain_lock = gain_lock;
        self
    }

    /// Activate gain auto_locking, when the target reference power is reached
    pub fn auto_lock(mut self, auto_lock: bool) -> AgcBuilder<T> {
        self.auto_lock = auto_lock;
        self
    }

    /// Create [`Agc`] block
    pub fn build(self) -> Agc<T> {
        Agc::<T>::new(
            self.squelch,
            self.max_gain,
            self.gain,
            self.adjustment_rate,
            self.reference_power,
            self.gain_lock,
            self.auto_lock,
        )
    }
}

impl<T: Send + Sync + ComplexFloat<Real: ToPrimitive> + Default + std::fmt::Debug + Copy + 'static>
    Default for AgcBuilder<T>
{
    fn default() -> Self {
        Self::new()
    }
}
