//! ## Blocks related to type conversion
//!
//! Conversion could be
//!  * exact, ie just like mathematical numbers
//!  * scaled, ie adjusted to range over the targeted type interval
//!  * lossy, ie with precision loss
//!
//! # Usage
//!
//! `u8` [0..255] will be converted into `f32` as [-1.0..1.0] with scaled conversion:
//! ```
//! # use fsdr_blocks::type_converters::TypeConvertersBuilder;
//! let blk = TypeConvertersBuilder::scale_convert::<u8, f32>().build();
//! ```
//!
//! `u8` [0..255] will be converted into `f32` as [0.0..255.0] with plain conversion:
//! ```
//! # use fsdr_blocks::type_converters::TypeConvertersBuilder;
//! let blk = TypeConvertersBuilder::convert::<u8, f32>().build();
//! ```

use core::marker::PhantomData;
#[cfg(feature = "simd")]
use core::simd::prelude::*;
use futuresdr::prelude::*;

/// Main builder for type conversion blocks
pub struct TypeConvertersBuilder {}

pub struct ConverterBuilder<A, B> {
    marker_input: PhantomData<A>,
    marker_output: PhantomData<B>,
    scaled: bool,
}

impl TypeConvertersBuilder {
    /// Exact conversion
    pub fn convert<A, B>() -> ConverterBuilder<A, B>
    where
        A: Copy + Send,
        B: Copy + Send + From<A>,
    {
        ConverterBuilder::<A, B> {
            marker_input: PhantomData,
            marker_output: PhantomData,
            scaled: false,
        }
    }

    /// Full range conversion
    /// for example u8 [0..255] will be converted into f32 as [-1.0..1.0]
    pub fn scale_convert<A, B>() -> ConverterBuilder<A, B> {
        ConverterBuilder::<A, B> {
            marker_input: PhantomData,
            marker_output: PhantomData,
            scaled: true,
        }
    }

    pub fn lossy_scale_convert_f32_u8() -> ConverterBuilder<f32, u8> {
        ConverterBuilder::<f32, u8> {
            marker_input: PhantomData,
            marker_output: PhantomData,
            scaled: true,
        }
    }

    pub fn lossy_scale_convert_f32_i8() -> ConverterBuilder<f32, i8> {
        ConverterBuilder::<f32, i8> {
            marker_input: PhantomData,
            marker_output: PhantomData,
            scaled: true,
        }
    }

    pub fn lossy_scale_convert_f32_i16() -> ConverterBuilder<f32, i16> {
        ConverterBuilder::<f32, i16> {
            marker_input: PhantomData,
            marker_output: PhantomData,
            scaled: true,
        }
    }
}

#[derive(Block)]
pub struct TypeConverter<
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    B: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O: CpuBufferWriter<Item = B> = DefaultCpuWriter<B>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    scaled: bool,
}

impl<A, B, I, O> TypeConverter<A, B, I, O>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    B: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = B>,
{
    pub fn new(scaled: bool) -> Self {
        Self {
            input: I::default(),
            output: O::default(),
            scaled,
        }
    }
}

pub trait TypeConvertSupported<B>: Copy {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [B]);
    fn convert_item(scaled: bool, item: &Self) -> B;
}

macro_rules! impl_type_convert_plain {
    ($($a:ty => $b:ty),*) => {
        $(
            impl TypeConvertSupported<$b> for $a {
                fn convert_slice(_scaled: bool, input: &[Self], output: &mut [$b]) {
                    let n = input.len().min(output.len());
                    for i in 0..n {
                        output[i] = <$b>::from(input[i]);
                    }
                }

                fn convert_item(_scaled: bool, item: &Self) -> $b {
                    <$b>::from(*item)
                }
            }
        )*
    };
}

impl_type_convert_plain!(
    u8 => u16, u8 => u32, u8 => u64,
    u16 => u32, u16 => u64,
    u32 => u64,
    i8 => i16, i8 => i32, i8 => i64,
    i16 => i32, i16 => i64,
    i32 => i64,
    f32 => f64,
    u8 => f64,
    u16 => f64,
    u32 => f64,
    i8 => f64,
    i16 => f64,
    i32 => f64
);

// Special case for f32 which don't have From<u32/i32>
impl TypeConvertSupported<f32> for u32 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [f32]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> f32 {
        if scaled {
            (*item as f32) / ((u32::MAX as f32) / 2.0) - 1.0
        } else {
            *item as f32
        }
    }
}

impl TypeConvertSupported<f32> for i32 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [f32]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> f32 {
        if scaled {
            (*item as f32) / ((i32::MAX as f32) / 2.0) - 1.0
        } else {
            *item as f32
        }
    }
}

// Types supported by scaled convert to f32
impl TypeConvertSupported<f32> for u8 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [f32]) {
        let n = input.len().min(output.len());
        if scaled {
            #[cfg(feature = "simd")]
            {
                const LANES: usize = 8;
                let n_simd = n / LANES;
                let offset = f32x8::splat(1.0);
                let scale = f32x8::splat(2.0 / 255.0);

                for i in 0..n_simd {
                    let v = u8x8::from_slice(&input[i * LANES..(i + 1) * LANES]);
                    let v_f32 = v.cast::<f32>();
                    let res = v_f32 * scale - offset;
                    res.copy_to_slice(&mut output[i * LANES..(i + 1) * LANES]);
                }

                for i in (n_simd * LANES)..n {
                    output[i] = Self::convert_item(true, &input[i]);
                }
            }
            #[cfg(not(feature = "simd"))]
            {
                for i in 0..n {
                    output[i] = Self::convert_item(true, &input[i]);
                }
            }
        } else {
            for i in 0..n {
                output[i] = input[i] as f32;
            }
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> f32 {
        if scaled {
            (*item as f32) * (2.0 / 255.0) - 1.0
        } else {
            *item as f32
        }
    }
}

impl TypeConvertSupported<f32> for u16 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [f32]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> f32 {
        if scaled {
            (*item as f32) / ((u16::MAX as f32) / 2.0) - 1.0
        } else {
            *item as f32
        }
    }
}

impl TypeConvertSupported<f32> for i8 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [f32]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> f32 {
        if scaled {
            (*item as f32) / ((i8::MAX as f32) / 2.0) - 1.0
        } else {
            *item as f32
        }
    }
}

impl TypeConvertSupported<f32> for i16 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [f32]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> f32 {
        if scaled {
            (*item as f32) / ((i16::MAX as f32) / 2.0) - 1.0
        } else {
            *item as f32
        }
    }
}

// f32 to integer (scaled)
impl TypeConvertSupported<u8> for f32 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [u8]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> u8 {
        if scaled {
            (*item * (u8::MAX as f32) * 0.5 + 128.0) as u8
        } else {
            *item as u8
        }
    }
}

impl TypeConvertSupported<i8> for f32 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [i8]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> i8 {
        if scaled {
            (*item * (i8::MAX as f32)) as i8
        } else {
            *item as i8
        }
    }
}

impl TypeConvertSupported<i16> for f32 {
    fn convert_slice(scaled: bool, input: &[Self], output: &mut [i16]) {
        let n = input.len().min(output.len());
        for i in 0..n {
            output[i] = Self::convert_item(scaled, &input[i]);
        }
    }
    fn convert_item(scaled: bool, item: &Self) -> i16 {
        if scaled {
            (*item * (i16::MAX as f32)) as i16
        } else {
            *item as i16
        }
    }
}

#[doc(hidden)]
impl<A, B, I, O> Kernel for TypeConverter<A, B, I, O>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy + TypeConvertSupported<B>,
    B: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = B>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();

        let n = i.len().min(o.len());
        if n > 0 {
            A::convert_slice(self.scaled, &i[..n], &mut o[..n]);
            self.input.consume(n);
            self.output.produce(n);
        }

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}

impl<A, B> ConverterBuilder<A, B>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy + TypeConvertSupported<B>,
    B: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
{
    pub fn build(self) -> TypeConverter<A, B> {
        TypeConverter::new(self.scaled)
    }
}
