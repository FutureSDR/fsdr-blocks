use core::marker::PhantomData;

use futuresdr::blocks::Apply;
use futuresdr::runtime::Block;

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
        Apply::new(|i: &u8| -> f32 { (*i as f32) / ((u8::MAX as f32) / 2.0) - 1.0 })
    }
}

impl ScaledConverterBuilder<i8, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &i8| -> f32 { (*i as f32) / ((i8::MAX as f32) / 2.0) - 1.0 })
    }
}

impl ScaledConverterBuilder<i16, f32> {
    pub fn build(self) -> Block {
        Apply::new(|i: &i16| -> f32 { (*i as f32) / ((i16::MAX as f32) / 2.0) - 1.0 })
    }
}

impl ScaledConverterBuilder<f32, u8> {
    pub fn build(self) -> Block {
        Apply::new(|i: &f32| -> u8 { (*i * (u8::MAX as f32) * 0.5 + 128.0) as u8 })
    }
}

impl ScaledConverterBuilder<f32, i8> {
    pub fn build(self) -> Block {
        Apply::new(|i: &f32| -> i8 { (*i * (i8::MAX as f32)) as i8 })
    }
}

impl ScaledConverterBuilder<f32, i16> {
    pub fn build(self) -> Block {
        Apply::new(|i: &f32| -> i16 { (*i * (i16::MAX as f32)) as i16 })
    }
}
