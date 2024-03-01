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
//!
//! Some other conversions are lossy because there is no natural conversion of all possible inputs.
//! Conversion of `f32` into `i6` is an example because `16.3` has no direct conversion, yet `16` is a good candidate.
//! But `f32` can also represent positive or negative infinity, and NaN (not a number) that are not convertible.
//!
//! ```
//! # use fsdr_blocks::type_converters::TypeConvertersBuilder;
//! let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i16().build();
//! ```

use core::marker::PhantomData;

use futuresdr::blocks::Apply;
use futuresdr::runtime::Block;

/// Main builder for type conversion blocks
pub struct TypeConvertersBuilder {}

pub struct ConverterBuilder<A, B> {
    marker_input: PhantomData<A>,
    marker_output: PhantomData<B>,
}

pub struct ScaledConverterBuilder<A, B> {
    marker_input: PhantomData<A>,
    marker_output: PhantomData<B>,
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
        }
    }

    /// Full range conversion
    /// for example u8 [0..255] will be converted into f32 as [-1.0..1.0]
    pub fn scale_convert<A, B>() -> ScaledConverterBuilder<A, B>
    where
        A: Copy + Send,
        B: Copy + Send + From<A>,
    {
        ScaledConverterBuilder::<A, B> {
            marker_input: PhantomData,
            marker_output: PhantomData,
        }
    }

    pub fn lossy_scale_convert_f32_u8() -> ScaledConverterBuilder<f32, u8> {
        ScaledConverterBuilder::<f32, u8> {
            marker_input: PhantomData,
            marker_output: PhantomData,
        }
    }

    pub fn lossy_scale_convert_f32_i8() -> ScaledConverterBuilder<f32, i8> {
        ScaledConverterBuilder::<f32, i8> {
            marker_input: PhantomData,
            marker_output: PhantomData,
        }
    }

    pub fn lossy_scale_convert_f32_i16() -> ScaledConverterBuilder<f32, i16> {
        ScaledConverterBuilder::<f32, i16> {
            marker_input: PhantomData,
            marker_output: PhantomData,
        }
    }
}

impl<A, B> ConverterBuilder<A, B>
where
    A: Copy + Send + 'static,
    B: Copy + Send + From<A> + 'static,
{
    pub fn build(self) -> Block {
        Apply::new(|i: &A| -> B { (*i).into() })
    }
}

impl ScaledConverterBuilder<u8, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &u8| -> f32 { ScaledConverterBuilder::<u8, f32>::convert(i) })
    }

    pub fn convert(i: &u8) -> f32 {
        (*i as f32) / ((u8::MAX as f32) / 2.0) - 1.0
    }
}

impl ScaledConverterBuilder<u16, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &u16| -> f32 { ScaledConverterBuilder::<u16, f32>::convert(i) })
    }

    pub fn convert(i: &u16) -> f32 {
        (*i as f32) / ((u16::MAX as f32) / 2.0) - 1.0
    }
}

impl ScaledConverterBuilder<u32, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &u32| -> f32 { ScaledConverterBuilder::<u32, f32>::convert(i) })
    }

    pub fn convert(i: &u32) -> f32 {
        (*i as f32) / ((u32::MAX as f32) / 2.0) - 1.0
    }
}

impl ScaledConverterBuilder<i8, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &i8| -> f32 { ScaledConverterBuilder::<i8, f32>::convert(i) })
    }

    pub fn convert(i: &i8) -> f32 {
        (*i as f32) / ((i8::MAX as f32) / 2.0) - 1.0
    }
}

impl ScaledConverterBuilder<i16, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &i16| -> f32 { ScaledConverterBuilder::<i16, f32>::convert(i) })
    }

    pub fn convert(i: &i16) -> f32 {
        (*i as f32) / ((i16::MAX as f32) / 2.0) - 1.0
    }
}

impl ScaledConverterBuilder<i32, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &i32| -> f32 { ScaledConverterBuilder::<i32, f32>::convert(i) })
    }

    pub fn convert(i: &i32) -> f32 {
        (*i as f32) / ((i32::MAX as f32) / 2.0) - 1.0
    }
}

impl ScaledConverterBuilder<f32, u8> {
    pub fn build(self) -> Block {
        Apply::new(|i: &f32| -> u8 { ScaledConverterBuilder::<f32, u8>::convert(i) })
    }

    pub fn convert(i: &f32) -> u8 {
        (*i * (u8::MAX as f32) * 0.5 + 128.0) as u8
    }
}

impl ScaledConverterBuilder<f32, i8> {
    pub fn build(self) -> Block {
        Apply::new(|i: &f32| -> i8 { ScaledConverterBuilder::<f32, i8>::convert(i) })
    }

    pub fn convert(i: &f32) -> i8 {
        (*i * (i8::MAX as f32)) as i8
    }
}

impl ScaledConverterBuilder<f32, i16> {
    pub fn build(self) -> Block {
        Apply::new(|i: &f32| -> i16 { ScaledConverterBuilder::<f32, i16>::convert(i) })
    }

    pub fn convert(i: &f32) -> i16 {
        (*i * (i16::MAX as f32)) as i16
    }
}
