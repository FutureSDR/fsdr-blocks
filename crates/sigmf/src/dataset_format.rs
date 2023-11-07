use std::fmt;

#[cfg(test)]
use quickcheck::{empty_shrinker, single_shrinker, Arbitrary, Gen};

use crate::SigMFError;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DatasetFormat {
    #[serde(rename="rf64_le")]
    Rf64Le,
    #[serde(rename="rf64_be")]
    Rf64Be,
    #[serde(rename="cf64_le")]
    Cf64Le,
    #[serde(rename="cf64_be")]
    Cf64Be,
    #[serde(rename="rf32_le")]
    Rf32Le,
    #[serde(rename="rf32_be")]
    Rf32Be,
    #[serde(rename="cf32_le")]
    Cf32Le,
    #[serde(rename="cf32_be")]
    Cf32Be,
    #[serde(rename="ri32_le")]
    Ri32Le,
    #[serde(rename="ri32_be")]
    Ri32Be,
    #[serde(rename="ci32_le")]
    Ci32Le,
    #[serde(rename="ci32_be")]
    Ci32Be,
    #[serde(rename="ri16_le")]
    Ri16Le,
    #[serde(rename="ri16_be")]
    Ri16Be,
    #[serde(rename="ci16_le")]
    Ci16Le,
    #[serde(rename="ci16_be")]
    Ci16Be,
    #[serde(rename="ru32_le")]
    Ru32Le,
    #[serde(rename="ru32_be")]
    Ru32Be,
    #[serde(rename="cu32_le")]
    Cu32Le,
    #[serde(rename="cu32_be")]
    Cu32Be,
    #[serde(rename="ru16_le")]
    Ru16Le,
    #[serde(rename="ru16_be")]
    Ru16Be,
    #[serde(rename="cu16_le")]
    Cu16Le,
    #[serde(rename="cu16_be")]
    Cu16Be,
    #[serde(rename="ri8")]
    RI8,
    #[serde(rename="ru8")]
    RU8,
    #[serde(rename="ci8")]
    CI8,
    #[serde(rename="cu8")]
    CU8,
}

impl DatasetFormat {
    /// The size in bits
    pub fn bits(&self) -> usize {
        use DatasetFormat::*;
        match self {
            Cf64Le | Cf64Be => 2 * 64,
            Rf64Le | Rf64Be => 64,

            Rf32Le | Rf32Be => 32,
            Cf32Le | Cf32Be => 2 * 32,

            Ri32Le | Ri32Be => 32,
            Ci32Le | Ci32Be => 2 * 32,

            Ri16Le | Ri16Be => 16,
            Ci16Le | Ci16Be => 2 * 16,

            Ru32Le | Ru32Be => 32,
            Cu32Le | Cu32Be => 2 * 32,

            Ru16Le | Ru16Be => 16,
            Cu16Le | Cu16Be => 2 * 16,

            CI8 => 2 * 8,
            CU8 => 2 * 8,
            RI8 => 8,
            RU8 => 8,
        }
    }

    /// The size in bytes
    pub fn size(&self) -> usize {
        self.bits() / 8
    }

    pub fn is_real(&self) -> bool {
        use DatasetFormat::*;
        match &self {
            Cf64Le | Cf64Be | Cf32Le | Cf32Be | Ci32Le | Ci32Be | Ci16Le | Ci16Be
            | Cu32Le | Cu32Be | Cu16Le | Cu16Be | CI8 | CU8 => false,

            Rf64Le | Rf64Be | Rf32Le | Rf32Be | Ri32Le | Ri32Be | Ri16Le | Ri16Be
            | Ru32Le | Ru32Be | Ru16Le | Ru16Be | RI8 | RU8 => true,
        }
    }

    pub fn is_complex(&self) -> bool {
        !self.is_real()
    }

    pub fn is_signed(&self) -> bool {
        use DatasetFormat::*;
        match self {
            Rf64Le | Rf64Be | Cf64Le | Cf64Be | Rf32Le | Rf32Be | Cf32Le | Cf32Be
            | Ri32Le | Ri32Be | Ci32Le | Ci32Be | Ri16Le | Ri16Be | Ci16Le | Ci16Be
            | RI8 | CI8 => true,

            Ru32Le | Ru32Be | Cu32Le | Cu32Be | Ru16Le | Ru16Be | Cu16Le | Cu16Be | RU8
            | CU8 => false,
        }
    }

    pub fn is_unsigned(&self) -> bool {
        !self.is_signed()
    }

    pub fn is_little_endian(&self) -> bool {
        use DatasetFormat::*;
        match self {
            Rf64Le | Cf64Le | Rf32Le | Cf32Le | Ri32Le | Ci32Le | Ri16Le | Ci16Le
            | Ru32Le | Cu32Le | Ru16Le | Cu16Le => true,
            _ => false,
        }
    }

    pub fn is_big_endian(&self) -> bool {
        use DatasetFormat::*;
        match self {
            Rf64Be | Cf64Be | Rf32Be | Cf32Be | Ri32Be | Ci32Be | Ri16Be | Ci16Be
            | Ru32Be | Cu32Be | Ru16Be | Cu16Be => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        use DatasetFormat::*;
        match self {
            Rf64Le | Rf64Be | Cf64Le | Cf64Be | Rf32Le | Rf32Be | Cf32Le | Cf32Be => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        !self.is_float()
    }

    pub fn is_byte(&self) -> bool {
        use DatasetFormat::*;
        match self {
            RI8 | CU8 | CI8 | RU8 => true,
            _ => false,
        }
    }

    pub fn all() -> [&'static DatasetFormat; 28] {
        use DatasetFormat::*;
        let alls = [
            &Rf64Le,
            &Rf64Be,
            &Cf64Le,
            &Cf64Be,
            &Rf32Le,
            &Rf32Be,
            &Cf32Le,
            &Cf32Be,
            &Ri32Le,
            &Ri32Be,
            &Ci32Le,
            &Ci32Be,
            &Ri16Le,
            &Ri16Be,
            &Ci16Le,
            &Ci16Be,
            &Ru32Le,
            &Ru32Be,
            &Cu32Le,
            &Cu32Be,
            &Ru16Le,
            &Ru16Be,
            &Cu16Le,
            &Cu16Be,
            &CI8,
            &CU8,
            &RI8,
            &RU8,
        ];
        alls
    }
}

impl fmt::Display for DatasetFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DatasetFormat::*;
        match self {
            Rf64Le => write!(f, "rf64_le"),
            Rf64Be => write!(f, "rf64_be"),
            Cf64Le => write!(f, "cf64_le"),
            Cf64Be => write!(f, "cf64_be"),
            Rf32Le => write!(f, "rf32_le"),
            Rf32Be => write!(f, "rf32_be"),
            Cf32Le => write!(f, "cf32_le"),
            Cf32Be => write!(f, "cf32_be"),
            Ri32Le => write!(f, "ri32_le"),
            Ri32Be => write!(f, "ri32_be"),
            Ci32Le => write!(f, "ci32_le"),
            Ci32Be => write!(f, "ci32_be"),
            Ri16Le => write!(f, "ri16_le"),
            Ri16Be => write!(f, "ri16_be"),
            Ci16Le => write!(f, "ci16_le"),
            Ci16Be => write!(f, "ci16_be"),
            Ru32Le => write!(f, "ru32_le"),
            Ru32Be => write!(f, "ru32_be"),
            Cu32Le => write!(f, "cu32_le"),
            Cu32Be => write!(f, "cu32_be"),
            Ru16Le => write!(f, "ru16_le"),
            Ru16Be => write!(f, "ru16_be"),
            Cu16Le => write!(f, "cu16_le"),
            Cu16Be => write!(f, "cu16_be"),
            CI8 => write!(f, "ci8"),
            CU8 => write!(f, "cu8"),
            RI8 => write!(f, "ri8"),
            RU8 => write!(f, "ru8"),
        }
    }
}

impl std::str::FromStr for DatasetFormat {
    type Err = SigMFError;
    fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
        use DatasetFormat::*;
        match s.to_lowercase().as_str() {
            "rf64_le" => Ok(Rf64Le),
            "rf64_be" => Ok(Rf64Be),
            "cf64_le" => Ok(Cf64Le),
            "cf64_be" => Ok(Cf64Be),
            "rf32_le" => Ok(Rf32Le),
            "rf32_be" => Ok(Rf32Be),
            "cf32_le" => Ok(Cf32Le),
            "cf32_be" => Ok(Cf32Be),
            "ri32_le" => Ok(Ri32Le),
            "ri32_be" => Ok(Ri32Be),
            "ci32_le" => Ok(Ci32Le),
            "ci32_be" => Ok(Ci32Be),
            "ri16_le" => Ok(Ri16Le),
            "ri16_be" => Ok(Ri16Be),
            "ci16_le" => Ok(Ci16Le),
            "ci16_be" => Ok(Ci16Be),
            "ru32_le" => Ok(Ru32Le),
            "ru32_be" => Ok(Ru32Be),
            "cu32_le" => Ok(Cu32Le),
            "cu32_be" => Ok(Cu32Be),
            "ru16_le" => Ok(Ru16Le),
            "ru16_be" => Ok(Ru16Be),
            "cu16_le" => Ok(Cu16Le),
            "cu16_be" => Ok(Cu16Be),
            "ri8" => Ok(RI8),
            "ru8" => Ok(RU8),
            "ci8" => Ok(CI8),
            "cu8" => Ok(CU8),
            _ => Err(SigMFError::UnknownDatasetFormat(s.to_string())),
        }
    }
}

#[cfg(test)]
impl Arbitrary for DatasetFormat {
    fn arbitrary(g: &mut Gen) -> DatasetFormat {
        **g.choose(&DatasetFormat::all()).unwrap()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use DatasetFormat::*;
        match self {
            Rf64Le => single_shrinker(Rf32Le),
            Rf64Be => single_shrinker(Rf32Be),
            Cf64Le => single_shrinker(Cf32Le),
            Cf64Be => single_shrinker(Cf32Be),
            Rf32Le => single_shrinker(Ri32Le),
            Rf32Be => single_shrinker(Rf32Le),
            Cf32Le => single_shrinker(Rf32Le),
            Cf32Be => single_shrinker(Rf32Be),
            Ri32Le => single_shrinker(Ri16Le),
            Ri32Be => single_shrinker(Ri16Be),
            Ci32Le => single_shrinker(Ci16Le),
            Ci32Be => single_shrinker(Ci16Be),
            Ri16Le => single_shrinker(RI8),
            Ri16Be => single_shrinker(RI8),
            Ci16Le => single_shrinker(CI8),
            Ci16Be => single_shrinker(CI8),
            Ru32Le => single_shrinker(Ru16Le),
            Ru32Be => single_shrinker(Ru16Be),
            Cu32Le => single_shrinker(Ru32Le),
            Cu32Be => single_shrinker(Ru32Be),
            Ru16Le => single_shrinker(RU8),
            Ru16Be => single_shrinker(RU8),
            Cu16Le => single_shrinker(CU8),
            Cu16Be => single_shrinker(CU8),
            CI8 => single_shrinker(CU8),
            CU8 => single_shrinker(RU8),
            RI8 => single_shrinker(RU8),
            RU8 => empty_shrinker(),
        }
    }
}

#[cfg(test)]
#[quickcheck]
fn qc_little_endian_ends_with_le(dataset: DatasetFormat) -> bool {
   !dataset.is_little_endian() | dataset.to_string().ends_with("_le")
}

#[cfg(test)]
#[quickcheck]
fn qc_big_endian_ends_with_be(dataset: DatasetFormat) -> bool {
   !dataset.is_big_endian() | dataset.to_string().ends_with("_be")
}

#[cfg(test)]
#[quickcheck]
fn qc_complex_starts_with_c(dataset: DatasetFormat) -> bool {
   !dataset.is_complex() | dataset.to_string().starts_with("c")
}

#[cfg(test)]
#[quickcheck]
fn qc_real_starts_with_r(dataset: DatasetFormat) -> bool {
   !dataset.is_real() | dataset.to_string().starts_with("r")
}

#[cfg(test)]
#[quickcheck]
fn qc_bits_in_label(dataset: DatasetFormat) -> bool {
    let mut nb_bits = dataset.bits();
    if dataset.is_complex() {
        nb_bits = nb_bits / 2;
    }
    let nb_bits = nb_bits.to_string().clone();
    let label = dataset.to_string();
    label.contains(nb_bits.as_str())
}

#[cfg(test)]
#[quickcheck]
fn qc_parse_string_is_identity(dataset: DatasetFormat) -> bool {
   let dataset_repr = dataset.to_string();
   let parsed = dataset_repr.parse::<DatasetFormat>();
   parsed.is_err() || parsed.unwrap() == dataset
}

pub struct DatasetFormatBuilder {}

impl DatasetFormatBuilder {
    // pub fn complex() -> DatasetFormatBuilder {

    // }

    // pub fn real() -> DatasetFormatBuilder {

    // }
}
