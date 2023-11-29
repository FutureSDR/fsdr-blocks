use futuresdr::blocks::VectorSink;

use fsdr_blocks::sigmf::{BytesConveter, SigMFSink, SigMFSourceBuilder};
use futuresdr::{
    anyhow::Result,
    blocks::{VectorSinkBuilder, VectorSource},
    macros::connect,
    runtime::{Flowgraph, Runtime},
};
use sigmf::{DatasetFormat, DescriptionBuilder};

/// Write the data into a SigMF file,
/// then read it back again
/// and compare data
fn sigmf_write_read<T>(datatype: DatasetFormat, data: Vec<T>) -> Result<()>
where
    T: Sized
        + 'static
        + Clone
        + std::marker::Send
        + std::marker::Sync
        + std::fmt::Debug
        + std::cmp::PartialEq,
    DatasetFormat: BytesConveter<T>,
{
    let mut fg = Flowgraph::new();

    let src1 = VectorSource::new(data.clone());
    let data_file_content: Vec<u8> = vec![];
    let meta_file_content: Vec<u8> = vec![];
    let data_file = std::io::Cursor::new(data_file_content);
    let meta_file = std::io::Cursor::new(meta_file_content);
    let desc = DescriptionBuilder::from(datatype);
    let snk1 = SigMFSink::<T, _, _>::new(data_file, desc, meta_file);
    connect!(fg,
        src1 > snk1;
    );
    fg = Runtime::new().run(fg)?;
    let snk1 = fg
        .kernel::<SigMFSink<T, std::io::Cursor<Vec<u8>>, std::io::Cursor<Vec<u8>>>>(snk1)
        .unwrap();
    let desc = snk1.description.build()?;
    let mut fg = Flowgraph::new();
    let data_file = snk1.writer.to_owned().into_inner();
    let data_file = futuresdr::futures::io::Cursor::new(data_file);
    let src2 = futuresdr::futures::executor::block_on(
        SigMFSourceBuilder::with_data_and_description(data_file, desc).build::<T>(),
    )?;
    let snk2 = VectorSinkBuilder::<T>::new().build();
    connect!(fg,
        src2 > snk2;
    );
    fg = Runtime::new().run(fg)?;
    let snk2 = fg.kernel::<VectorSink<T>>(snk2).unwrap().items();
    assert_eq!(data.len(), snk2.len());
    for (o, i) in data.iter().zip(snk2) {
        assert_eq!(*o, *i);
    }
    Ok(())
}

#[test]
fn sigmf_write_read_ru8() -> Result<()> {
    let data = vec![6u8, 8, 10, 12];
    let datatype = DatasetFormat::RU8;
    sigmf_write_read(datatype, data)
}

#[test]
fn sigmf_write_read_ri8() -> Result<()> {
    let data = vec![6i8, 8, -10, 0, 12];
    let datatype = DatasetFormat::RI8;
    sigmf_write_read(datatype, data)
}
