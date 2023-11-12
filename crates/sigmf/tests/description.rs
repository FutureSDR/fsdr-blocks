#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::io::Cursor;

use sigmf::{DatasetFormat, Description, DescriptionBuilder, SigMFError};

#[quickcheck]
fn qc_write_read(desc_input: Description) -> bool {
    let mut buffer = Vec::<u8>::new();
    if let Err(_) = desc_input.to_writer(&mut buffer) {
        return false;
    }
    let buffer = Cursor::new(buffer);
    let desc_output = Description::from_reader(buffer).expect("");
    assert_eq!(
        desc_input.global().expect("").sample_rate,
        desc_output.global().expect("").sample_rate
    );
    desc_input == desc_output
}

#[test]
fn create_desc_high_sample_rate() -> Result<(), SigMFError> {
    let mut desc = DescriptionBuilder::from(DatasetFormat::Cf32Le);
    let setter_ok = desc.sample_rate(2.7350335256693894e251);
    assert!(setter_ok.is_err());
    let setter_ok: Result<&mut DescriptionBuilder, SigMFError> = desc.sample_rate(f64::NAN);
    assert!(setter_ok.is_err());
    let setter_ok = desc.sample_rate(2_000_000.0);
    assert!(setter_ok.is_ok());
    Ok(())
}
