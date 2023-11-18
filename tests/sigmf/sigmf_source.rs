use fsdr_blocks::sigmf::SigMFSource;
use futuresdr::anyhow::Result;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
use futuresdr::futures::io::BufReader;
use futuresdr::futures::io::Cursor;
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use sigmf::DatasetFormat;
use sigmf::DescriptionBuilder;

fn test_no_conversion<T>(data: &[u8], datatype: DatasetFormat) -> Result<Vec<T>>
where
    T: Sized
        + std::marker::Send
        + std::marker::Sync
        + std::marker::Copy
        + std::fmt::Debug
        + 'static,
{
    let mut fg = Flowgraph::new();
    #[cfg(target_endian = "big")]
    let desc = DescriptionBuilder::from(DatasetFormat::Ru16Be).build()?;
    #[cfg(target_endian = "little")]
    let desc = DescriptionBuilder::from(datatype).build()?;

    let actual_file = Cursor::new(Vec::from(data));
    let actual_file = BufReader::new(actual_file);
    let src = SigMFSource::<T, _>::new(actual_file, desc);
    let snk = VectorSinkBuilder::<T>::new().build();

    connect!(fg,
        src > snk;
    );

    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<T>>(snk).unwrap();
    Ok(snk.items().clone())
}

#[test]
fn sigmf_source_u8_u8() -> Result<()> {
    let data = [6u8, 8, 10, 12];
    let datatype = DatasetFormat::RU8;
    let snk = test_no_conversion::<u8>(&data, datatype)?;
    let expected = vec![6u8, 8, 10, 12];
    assert_eq!(snk.len(), expected.len());
    for (o, i) in expected.iter().zip(snk) {
        assert_eq!(*o, i);
    }
    Ok(())
}

#[test]
fn sigmf_source_u16_u16() -> Result<()> {
    let data = [6u8, 8, 10, 12];
    #[cfg(target_endian = "big")]
    let datatype = DatasetFormat::Ru16Be;
    #[cfg(target_endian = "little")]
    let datatype = DatasetFormat::Ru16Le;
    let snk = test_no_conversion::<u16>(&data, datatype)?;
    let expected = vec![2054, 3082];
    assert_eq!(snk.len(), expected.len());
    for (o, i) in expected.iter().zip(snk) {
        assert_eq!(*o, i);
    }
    Ok(())
}

#[test]
fn sigmf_source_u32_u32() -> Result<()> {
    let data = [6u8, 8, 10, 12];
    #[cfg(target_endian = "big")]
    let datatype = DatasetFormat::Ru32Be;
    #[cfg(target_endian = "little")]
    let datatype = DatasetFormat::Ru32Le;
    let snk = test_no_conversion::<u32>(&data, datatype)?;
    let expected = vec![201984006];
    assert_eq!(snk.len(), expected.len());
    for (o, i) in expected.iter().zip(snk) {
        assert_eq!(*o, i);
    }
    Ok(())
}

#[test]
fn sigmf_source_cu16_cu16() -> Result<()> {
    let data = [6u8, 8, 10, 12];
    #[cfg(target_endian = "big")]
    let datatype = DatasetFormat::Cu16Be;
    #[cfg(target_endian = "little")]
    let datatype = DatasetFormat::Cu16Le;
    let snk = test_no_conversion::<Complex<u16>>(&data, datatype)?;
    let expected = vec![Complex::<u16> { re: 2054, im: 3082 }];
    assert_eq!(snk.len(), expected.len());
    for (o, i) in expected.iter().zip(snk) {
        assert_eq!(*o, i);
    }
    Ok(())
}
