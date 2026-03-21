mod sigmf_source;
use futuresdr::num_complex::Complex;
pub use sigmf_source::{SigMFSource, SigMFSourceBuilder};
mod sigmf_sink;
pub use sigmf::*;
pub use sigmf_sink::{SigMFSink, SigMFSinkBuilder};

use crate::type_converters::TypeConvertSupported;

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
            Rf32Le => f32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            Rf32Be => f32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            // Cf32Le => write!(f, "cf32_le"),
            // Cf32Be => write!(f, "cf32_be"),
            Ri32Le => i32::convert_item(true, &i32::from_le_bytes(bytes[0..4].try_into().unwrap())),
            Ri32Be => i32::convert_item(true, &i32::from_be_bytes(bytes[0..4].try_into().unwrap())),
            // Ci32Le => write!(f, "ci32_le"),
            // Ci32Be => write!(f, "ci32_be"),
            Ri16Le => i16::convert_item(true, &i16::from_le_bytes(bytes[0..2].try_into().unwrap())),
            Ri16Be => i16::convert_item(true, &i16::from_be_bytes(bytes[0..2].try_into().unwrap())),
            // Ci16Le => write!(f, "ci16_le"),
            // Ci16Be => write!(f, "ci16_be"),
            Ru32Le => u32::convert_item(true, &u32::from_le_bytes(bytes[0..4].try_into().unwrap())),
            Ru32Be => u32::convert_item(true, &u32::from_be_bytes(bytes[0..4].try_into().unwrap())),
            // Cu32Le => write!(f, "cu32_le"),
            // Cu32Be => write!(f, "cu32_be"),
            Ru16Le => u16::convert_item(true, &u16::from_le_bytes(bytes[0..2].try_into().unwrap())),
            Ru16Be => u16::convert_item(true, &u16::from_be_bytes(bytes[0..2].try_into().unwrap())),
            // Cu16Le => write!(f, "cu16_le"),
            // Cu16Be => write!(f, "cu16_be"),
            // CI8 => write!(f, "ci8"),
            // CU8 => write!(f, "cu8"),
            RI8 => i8::convert_item(true, &i8::from_ne_bytes(bytes[0..1].try_into().unwrap())),
            RU8 => u8::convert_item(true, &(bytes[0])),
            _ => todo!("not yet implemented"),
        }
    }
}

impl BytesConveter<u8> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> u8 {
        use DatasetFormat::*;
        match self {
            RU8 => bytes[0],
            _ => todo!("not yet implemented"),
        }
    }
}

impl BytesConveter<i8> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> i8 {
        use DatasetFormat::*;
        match self {
            RI8 => bytes[0] as i8,
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
            Ru32Le => u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            Ru32Be => u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            _ => todo!("not yet implemented"),
        }
    }
}

impl BytesConveter<Complex<u16>> for DatasetFormat {
    fn convert(self, bytes: &[u8]) -> Complex<u16> {
        use DatasetFormat::*;
        match self {
            Cu16Be => Complex::new(
                u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
            ),
            Cu16Le => Complex::new(
                u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
                u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
            ),
            Ru16Be => Complex::new(u16::from_be_bytes(bytes[0..2].try_into().unwrap()), 0u16),
            Ru16Le => Complex::new(u16::from_le_bytes(bytes[0..2].try_into().unwrap()), 0u16),
            _ => todo!("not yet implemented"),
        }
    }
}
