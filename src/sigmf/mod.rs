mod sigmf_source;
pub use sigmf_source::{SigMFSource, SigMFSourceBuilder};
mod sigmf_sink;
pub use sigmf::*;
pub use sigmf_sink::{SigMFSink, SigMFSinkBuilder};

use crate::type_converters::ScaledConverterBuilder;

pub trait BytesConveter<T>
where
    T: Sized,
{
    fn convert(self, bytes: &[u8]) -> T;
}

impl BytesConveter<f32> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> f32 {
        use DatasetFormat::*;
        match self {
            Rf64Le => f64::from_le_bytes(bytes[0..8].try_into().unwrap()) as f32,
            Rf64Be => f64::from_ne_bytes(bytes[0..8].try_into().unwrap()) as f32,
            // Cf64Le => write!(f, "cf64_le"),
            // Cf64Be => write!(f, "cf64_be"),
            Rf32Le => f32::from_le_bytes(bytes[0..4].try_into().unwrap()) as f32,
            Rf32Be => f32::from_be_bytes(bytes[0..4].try_into().unwrap()) as f32,
            // Cf32Le => write!(f, "cf32_le"),
            // Cf32Be => write!(f, "cf32_be"),
            Ri32Le => ScaledConverterBuilder::<i32, f32>::convert(&i32::from_le_bytes(
                bytes[0..4].try_into().unwrap(),
            )),
            Ri32Be => ScaledConverterBuilder::<i32, f32>::convert(&i32::from_be_bytes(
                bytes[0..4].try_into().unwrap(),
            )),
            // Ci32Le => write!(f, "ci32_le"),
            // Ci32Be => write!(f, "ci32_be"),
            Ri16Le => ScaledConverterBuilder::<i16, f32>::convert(&i16::from_le_bytes(
                bytes[0..2].try_into().unwrap(),
            )),
            Ri16Be => ScaledConverterBuilder::<i16, f32>::convert(&i16::from_be_bytes(
                bytes[0..2].try_into().unwrap(),
            )),
            // Ci16Le => write!(f, "ci16_le"),
            // Ci16Be => write!(f, "ci16_be"),
            Ru32Le => ScaledConverterBuilder::<u32, f32>::convert(&u32::from_le_bytes(
                bytes[0..4].try_into().unwrap(),
            )),
            Ru32Be => ScaledConverterBuilder::<u32, f32>::convert(&u32::from_be_bytes(
                bytes[0..4].try_into().unwrap(),
            )),
            // Cu32Le => write!(f, "cu32_le"),
            // Cu32Be => write!(f, "cu32_be"),
            Ru16Le => ScaledConverterBuilder::<u16, f32>::convert(&u16::from_le_bytes(
                bytes[0..2].try_into().unwrap(),
            )),
            Ru16Be => ScaledConverterBuilder::<u16, f32>::convert(&u16::from_be_bytes(
                bytes[0..2].try_into().unwrap(),
            )),
            // Cu16Le => write!(f, "cu16_le"),
            // Cu16Be => write!(f, "cu16_be"),
            // CI8 => write!(f, "ci8"),
            // CU8 => write!(f, "cu8"),
            RI8 => ScaledConverterBuilder::<i8, f32>::convert(&i8::from_ne_bytes(
                bytes[0..1].try_into().unwrap(),
            )),
            RU8 => ScaledConverterBuilder::<u8, f32>::convert(&(bytes[0] as u8)),
            _ => todo!("not yet implemented"),
        }
    }
}

impl BytesConveter<u8> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> u8 {
        use DatasetFormat::*;
        match self {
            RU8 => bytes[0] as u8,
            _ => todo!("not yet implemented"),
        }
    }
}

impl BytesConveter<u16> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> u16 {
        use DatasetFormat::*;
        match self {
            Ru16Le => u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            Ru16Be => u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
            _ => todo!("not yet implemented"),
        }
    }
}

impl BytesConveter<u32> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> u32 {
        use DatasetFormat::*;
        match self {
            Ru32Le => u32::from_le_bytes(bytes[0..2].try_into().unwrap()),
            Ru32Be => u32::from_be_bytes(bytes[0..2].try_into().unwrap()),
            _ => todo!("not yet implemented"),
        }
    }
}
