use futuresdr::anyhow::Result;
use futuresdr::runtime::Pmt;

use fsdr_blocks::serde_pmt::to_pmt;

#[test]
fn test_pmt_uint32() -> Result<()> {
    assert_eq!(Pmt::U32(42u32), to_pmt(&42u32)?);
    assert_eq!(Pmt::U32(u32::MAX), to_pmt(&u32::MAX)?);
    assert_eq!(Pmt::U32(u32::MIN), to_pmt(&u32::MIN)?);
    Ok(())
}

#[test]
fn test_pmt_none() -> Result<()> {
    assert_eq!(Pmt::Null, to_pmt(&Option::<u32>::None)?);
    Ok(())
}

#[test]
fn test_pmt_string() -> Result<()> {
    assert_eq!(Pmt::String("a string".to_string()), to_pmt(&"a string")?);
    assert_eq!(
        Pmt::String("a string".to_string()),
        to_pmt(&("a string".to_string()))?
    );
    assert_eq!(Pmt::String("".to_string()), to_pmt(&"")?);
    Ok(())
}

#[test]
fn test_pmt_bool() -> Result<()> {
    assert_eq!(Pmt::Bool(true), to_pmt(&true)?);
    assert_eq!(Pmt::Bool(false), to_pmt(&false)?);
    Ok(())
}
