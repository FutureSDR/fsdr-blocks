#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use sigmf::{DatasetFormat, DatasetFormatBuilder, SigMFError};

#[quickcheck]
fn qc_little_endian_ends_with_le(dataset: DatasetFormat) -> bool {
    !dataset.is_little_endian() | dataset.to_string().ends_with("_le")
}

#[quickcheck]
fn qc_big_endian_ends_with_be(dataset: DatasetFormat) -> bool {
    !dataset.is_big_endian() | dataset.to_string().ends_with("_be")
}

#[quickcheck]
fn qc_complex_starts_with_c(dataset: DatasetFormat) -> bool {
    !dataset.is_complex() | dataset.to_string().starts_with('c')
}

#[quickcheck]
fn qc_real_starts_with_r(dataset: DatasetFormat) -> bool {
    !dataset.is_real() | dataset.to_string().starts_with('r')
}

#[quickcheck]
fn qc_bits_in_label(dataset: DatasetFormat) -> bool {
    let mut nb_bits = dataset.bits();
    if dataset.is_complex() {
        nb_bits /= 2;
    }
    let nb_bits = nb_bits.to_string().clone();
    let label = dataset.to_string();
    label.contains(nb_bits.as_str())
}

#[quickcheck]
fn qc_parse_string_is_identity(dataset: DatasetFormat) -> bool {
    let dataset_repr = dataset.to_string();
    let parsed = dataset_repr.parse::<DatasetFormat>();
    parsed.is_err() || parsed.unwrap() == dataset
}

#[test]
fn test_dataset_builder() -> Result<(), SigMFError> {
    let datatype = DatasetFormatBuilder::<u32>::complex()
        .little_endian()
        .build();
    assert_eq!("cu32_le", datatype.to_string());
    let datatype = DatasetFormatBuilder::<f32>::real().big_endian().build();
    assert_eq!("rf32_be", datatype.to_string());
    Ok(())
}
