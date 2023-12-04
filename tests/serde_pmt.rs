use std::collections::HashMap;

use futuresdr::anyhow::Result;
use futuresdr::runtime::Pmt;

use fsdr_blocks::serde_pmt::to_pmt;
use serde_json;

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

#[test]
fn test_pmt_f32() -> Result<()> {
    assert_eq!(Pmt::F32(42.3f32), to_pmt(&42.3f32)?);
    assert_eq!(Pmt::F32(f32::MAX), to_pmt(&f32::MAX)?);
    assert_eq!(Pmt::F32(f32::MIN), to_pmt(&f32::MIN)?);
    Ok(())
}

#[test]
fn test_pmt_f64() -> Result<()> {
    assert_eq!(Pmt::F64(42.3f64), to_pmt(&42.3f64)?);
    assert_eq!(Pmt::F64(f64::MAX), to_pmt(&f64::MAX)?);
    assert_eq!(Pmt::F64(f64::MIN), to_pmt(&f64::MIN)?);
    Ok(())
}

#[test]
fn test_pmt_i16() -> Result<()> {
    assert_eq!(Pmt::F32(-3f32), to_pmt(&-3i16)?);
    assert_eq!(Pmt::F32(5f32), to_pmt(&5i16)?);
    assert_eq!(Pmt::F32(i16::MIN as f32), to_pmt(&i16::MIN)?);
    Ok(())
}

#[test]
fn test_pmt_char() -> Result<()> {
    assert_eq!(Pmt::String("a".to_string()), to_pmt(&'a')?);
    Ok(())
}

// TODO
// #[test]
// fn test_pmt_slice_u8() -> Result<()> {
//     let v = [8u8, 0, 5, 45, 255];
//     let expected: Vec<u8> = v.iter().map(|x| *x).collect();
//     assert_eq!(Pmt::Blob(expected), to_pmt(&v)?);

//     let v = vec![8u8, 9, 45, 26, 255, 0];
//     let expected = v.clone();
//     let v = &v[..];
//     assert_eq!(Pmt::Blob(expected), to_pmt(v)?);
//     Ok(())
// }

#[test]
fn test_pmt_option_char() -> Result<()> {
    assert_eq!(Pmt::String("a".to_string()), to_pmt(&Some('a'))?);
    assert_eq!(Pmt::Null, to_pmt(&Option::<char>::None)?);
    Ok(())
}

#[test]
fn test_pmt_sigmf_annot() -> Result<()> {
    let mut annot = sigmf::Annotation::default();
    annot.sample_start = Some(0);
    let mut expected = HashMap::new();
    expected.insert("core:sample_start".to_string(), Pmt::U64(0));
    assert_eq!(Pmt::MapStrPmt(expected.clone()), to_pmt(&annot)?);

    annot.comment = Some("a comment".to_string());
    expected.insert("core:comment".to_string(), to_pmt("a comment")?);
    assert_eq!(Pmt::MapStrPmt(expected.clone()), to_pmt(&annot)?);

    annot.extra.insert(
        "some_ext:some_field".to_string(),
        serde_json::to_value(456)?,
    );
    expected.insert("some_ext:some_field".to_string(), to_pmt(&456u64)?);
    assert_eq!(Pmt::MapStrPmt(expected.clone()), to_pmt(&annot)?);
    Ok(())
}
