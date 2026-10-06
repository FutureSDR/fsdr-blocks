//! ## Blocks related to type conversion
//!
//! Conversion could be:
//!  * exact, i.e. mathematical / lossless types conversion
//!  * scaled, i.e. mapped to/from the normalized `[-1.0, 1.0]` float range (zero-centered, following SDR / GNU Radio conventions)
//!  * lossy, i.e. with rounding or range-clamping
//!
//! # Usage
//!
//! `u8` [0..255] will be converted into `f32` as [-1.0..1.0] (with 128 mapping to 0.0):
//! ```
//! # use fsdr_blocks::type_converters::TypeConvertersBuilder;
//! let blk = TypeConvertersBuilder::scale_convert::<u8, f32>().build();
//! ```
//!
//! `i16` [-32768..32767] will be converted into `f32` as [-1.0..1.0] (with 0 mapping to 0.0):
//! ```
//! # use fsdr_blocks::type_converters::TypeConvertersBuilder;
//! let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
//! ```
//!
//! `u8` [0..255] will be converted into `f32` as [0.0..255.0] with plain/exact conversion:
//! ```
//! # use fsdr_blocks::type_converters::TypeConvertersBuilder;
//! let blk = TypeConvertersBuilder::convert::<u8, f32>().build();
//! ```

use core::marker::PhantomData;
use futuresdr::blocks::Apply;
use std::simd::prelude::*;

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
    /// Exact conversion (lossless or direct `From<A>` mapping)
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

    /// Full range, zero-centered conversion (e.g. `i16` [-32768..32767] to `f32` [-1.0..1.0])
    pub fn scale_convert<A, B>() -> ScaledConverterBuilder<A, B>
    where
        A: Copy + Send,
        B: Copy + Send,
    {
        ScaledConverterBuilder::<A, B> {
            marker_input: PhantomData,
            marker_output: PhantomData,
        }
    }

    pub fn lossy_scale_convert_f32_u8() -> ScaledConverterBuilder<f32, u8> {
        Self::scale_convert::<f32, u8>()
    }

    pub fn lossy_scale_convert_f32_i8() -> ScaledConverterBuilder<f32, i8> {
        Self::scale_convert::<f32, i8>()
    }

    pub fn lossy_scale_convert_f32_i16() -> ScaledConverterBuilder<f32, i16> {
        Self::scale_convert::<f32, i16>()
    }

    pub fn lossy_scale_convert_f32_i32() -> ScaledConverterBuilder<f32, i32> {
        Self::scale_convert::<f32, i32>()
    }

    pub fn lossy_scale_convert_f32_u16() -> ScaledConverterBuilder<f32, u16> {
        Self::scale_convert::<f32, u16>()
    }

    pub fn lossy_scale_convert_f32_u32() -> ScaledConverterBuilder<f32, u32> {
        Self::scale_convert::<f32, u32>()
    }
}

impl<A, B> ConverterBuilder<A, B>
where
    A: Copy + Send + Sync + Default + std::fmt::Debug + 'static,
    B: Copy + Send + Sync + Default + std::fmt::Debug + From<A> + 'static,
{
    pub fn build(self) -> Apply<impl FnMut(&A) -> B + Send + 'static, A, B> {
        Apply::new(|i: &A| -> B { (*i).into() })
    }
}

macro_rules! impl_scaled_converter {
    ($src:ty, $dst:ty, $conv:expr) => {
        impl ScaledConverterBuilder<$src, $dst> {
            #[inline(always)]
            pub fn build(self) -> Apply<impl FnMut(&$src) -> $dst + Send + 'static, $src, $dst> {
                Apply::new(|i: &$src| -> $dst { ScaledConverterBuilder::<$src, $dst>::convert(i) })
            }

            #[inline(always)]
            pub fn convert(i: &$src) -> $dst {
                let f: fn(&$src) -> $dst = $conv;
                f(i)
            }
        }
    };
}

// Signed integer <-> f32 conversions (zero-centered at 0.0, GNU Radio standard)
impl_scaled_converter!(i8, f32, |i| {
    f32::algebraic_mul(*i as f32, 1.0 / (i8::MAX as f32))
});
impl_scaled_converter!(i16, f32, |i| {
    f32::algebraic_mul(*i as f32, 1.0 / (i16::MAX as f32))
});
impl_scaled_converter!(i32, f32, |i| {
    f32::algebraic_mul(*i as f32, 1.0 / (i32::MAX as f32))
});

impl_scaled_converter!(f32, i8, |i| {
    let scaled = f32::algebraic_mul(*i, i8::MAX as f32);
    scaled.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8
});
impl ScaledConverterBuilder<f32, i16> {
    #[inline(always)]
    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn convert_slice(src: &[f32], dst: &mut [i16]) {
        let len = src.len().min(dst.len());
        let (src_chunks, src_rem) = src[..len].as_chunks::<8>();
        let (dst_chunks, dst_rem) = dst[..len].as_chunks_mut::<8>();

        let scale = Simd::splat(32767.0f32);
        let min_val = Simd::splat(-32768.0f32);
        let max_val = Simd::splat(32767.0f32);

        for (s, d) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
            let v = Simd::from_array(*s);
            let scaled = v * scale;
            let clamped = scaled.simd_clamp(min_val, max_val);
            let arr = clamped.to_array();
            for i in 0..8 {
                d[i] = arr[i].round() as i16;
            }
        }

        for (s, d) in src_rem.iter().zip(dst_rem.iter_mut()) {
            let scaled = f32::algebraic_mul(*s, 32767.0);
            *d = scaled.round().clamp(-32768.0, 32767.0) as i16;
        }
    }
}

impl_scaled_converter!(f32, i16, |i| {
    let scaled = f32::algebraic_mul(*i, i16::MAX as f32);
    scaled.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
});
impl_scaled_converter!(f32, i32, |i| {
    let scaled = f32::algebraic_mul(*i, i32::MAX as f32);
    scaled.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32
});

// Unsigned integer <-> f32 conversions (midpoint mapped to 0.0, GNU Radio standard)
impl_scaled_converter!(u8, f32, |i| {
    f32::algebraic_mul(f32::algebraic_sub(*i as f32, 128.0), 1.0 / 128.0)
});
impl_scaled_converter!(u16, f32, |i| {
    f32::algebraic_mul(f32::algebraic_sub(*i as f32, 32768.0), 1.0 / 32768.0)
});
impl_scaled_converter!(u32, f32, |i| {
    f64::algebraic_mul(
        f64::algebraic_sub(*i as f64, 2147483648.0),
        1.0 / 2147483648.0,
    ) as f32
});

impl_scaled_converter!(f32, u8, |i| {
    let scaled = (*i).mul_add(128.0, 128.0);
    scaled.round().clamp(0.0, 255.0) as u8
});
impl_scaled_converter!(f32, u16, |i| {
    let scaled = (*i).mul_add(32768.0, 32768.0);
    scaled.round().clamp(0.0, 65535.0) as u16
});
impl_scaled_converter!(f32, u32, |i| {
    let scaled = ((*i) as f64).mul_add(2147483648.0, 2147483648.0);
    scaled.round().clamp(0.0, u32::MAX as f64) as u32
});
