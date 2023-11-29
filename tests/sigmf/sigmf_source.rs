use fsdr_blocks::sigmf::BytesConveter;
use fsdr_blocks::sigmf::SigMFSourceBuilder;
use futuresdr::anyhow::Result;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
use futuresdr::futures::io::BufReader;
use futuresdr::futures::io::Cursor;
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use sigmf::DatasetFormat;
use sigmf::DescriptionBuilder;

pub fn test_no_conversion<T>(data: &[u8], datatype: DatasetFormat) -> Result<Vec<T>>
where
    T: Sized
        + std::marker::Send
        + std::marker::Sync
        + std::marker::Copy
        + std::fmt::Debug
        + 'static,
    DatasetFormat: BytesConveter<T>,
{
    let mut fg = Flowgraph::new();
    let desc = DescriptionBuilder::from(datatype).build()?;

    let actual_file = Cursor::new(Vec::from(data));
    let actual_file = BufReader::new(actual_file);
    let src = futuresdr::futures::executor::block_on(
        SigMFSourceBuilder::with_data_and_description(actual_file, desc).build::<T>(),
    )?;
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
    assert_eq!(expected.len(), snk.len());
    for (expected, actual) in expected.iter().zip(snk) {
        assert_eq!(*expected, actual);
    }
    Ok(())
}

#[test]
fn sigmf_source_u32_u32() -> Result<()> {
    let data = [6u8, 8, 10, 12].repeat(10);
    #[cfg(target_endian = "big")]
    let datatype = DatasetFormat::Ru32Be;
    #[cfg(target_endian = "little")]
    let datatype = DatasetFormat::Ru32Le;
    let snk = test_no_conversion::<u32>(&data, datatype)?;
    let expected = vec![201984006].repeat(10);
    assert_eq!(expected.len(), snk.len());
    for (o, i) in expected.iter().zip(snk) {
        assert_eq!(*o, i);
    }
    Ok(())
}

#[test]
fn sigmf_source_cu16_cu16() -> Result<()> {
    let data = [6u8, 8, 10, 12].repeat(4);
    #[cfg(target_endian = "big")]
    let datatype = DatasetFormat::Cu16Be;
    #[cfg(target_endian = "little")]
    let datatype = DatasetFormat::Cu16Le;
    let snk = test_no_conversion::<Complex<u16>>(&data, datatype)?;
    let expected = vec![Complex::<u16> { re: 2054, im: 3082 }].repeat(4);
    assert_eq!(expected.len(), snk.len());
    for (o, i) in expected.iter().zip(snk) {
        assert_eq!(*o, i);
    }
    Ok(())
}
