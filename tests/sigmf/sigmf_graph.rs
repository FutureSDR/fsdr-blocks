use futuresdr::blocks::VectorSink;

use fsdr_blocks::sigmf::{BytesConveter, SigMFSink, SigMFSourceBuilder};
use futuresdr::{
    blocks::{VectorSinkBuilder, VectorSource},
    macros::connect,
    runtime::Result,
    runtime::{Flowgraph, Runtime},
};

use futuresdr::futures::io::BufReader;
use futuresdr::futures::io::Cursor;
use sigmf::{Annotation, DatasetFormat, DescriptionBuilder};

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

#[test]
fn sigmf_read_write_annotation() -> Result<()> {
    let data = vec![6u8; 45];
    let datatype = DatasetFormat::RU8;
    let mut fg = Flowgraph::new();

    let mut desc = DescriptionBuilder::from(datatype);
    let annot = Annotation {
        label: Some("abc".to_string()),
        comment: Some("the comment".to_string()),
        sample_start: Some(10),
        sample_count: Some(20),
        ..Annotation::default()
    };
    desc.add_annotation(annot)?;
    let annot = Annotation {
        label: Some("the annot".to_string()),
        comment: Some("another comment".to_string()),
        sample_start: Some(15),
        sample_count: Some(25),
        ..Annotation::default()
    };
    desc.add_annotation(annot)?;
    let desc = desc.build()?;

    let actual_file = Cursor::new(data);
    let actual_file = BufReader::new(actual_file);
    let src1 = futuresdr::futures::executor::block_on(
        SigMFSourceBuilder::with_data_and_description(actual_file, desc).build::<u8>(),
    )?;

    let data_file_content: Vec<u8> = vec![];
    let meta_file_content: Vec<u8> = vec![];
    let data_file = std::io::Cursor::new(data_file_content);
    let meta_file = std::io::Cursor::new(meta_file_content);
    let tgt_desc = DescriptionBuilder::from(datatype);
    let snk1 = SigMFSink::<u8, _, _>::new(data_file, tgt_desc, meta_file);

    // Direct from source to sinkflowgraph
    connect!(fg,
        src1 > snk1;
    );
    // Now run the flowgraph
    fg = Runtime::new().run(fg)?;

    // Time to verify
    let snk1 = fg
        .kernel::<SigMFSink<u8, std::io::Cursor<Vec<u8>>, std::io::Cursor<Vec<u8>>>>(snk1)
        .unwrap();
    let tgt_desc = snk1.description.build()?;
    let annotations = tgt_desc.annotations()?;
    assert_eq!(2, annotations.len());
    let annot1 = annotations
        .first()
        .expect("the annotation should have been recreated");
    assert_eq!(
        "the comment",
        annot1
            .comment
            .as_ref()
            .expect("comment should have been copied")
            .as_str()
    );
    assert_eq!(
        "abc",
        annot1
            .label
            .as_ref()
            .expect("label should have been copied")
            .as_str()
    );

    let annot1 = annotations
        .first()
        .expect("the annotation should have been recreated");
    assert_eq!(
        "the comment",
        annot1
            .comment
            .as_ref()
            .expect("comment should have been copied")
            .as_str()
    );
    assert_eq!(
        "abc",
        annot1
            .label
            .as_ref()
            .expect("label should have been copied")
            .as_str()
    );

    Ok(())
}
