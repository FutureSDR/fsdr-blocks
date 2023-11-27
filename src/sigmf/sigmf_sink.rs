use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;

use futuresdr::anyhow::Result;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::runtime::{Block, Pmt, Tag};

use sigmf::{Annotation, Capture, Description};
use sigmf::{DatasetFormat, DatasetFormatBuilder, DescriptionBuilder, RecordingBuilder};

/// Write samples from a SigMF file.
///
/// # Inputs
///
/// `in`: input samples with tags annotations
///
/// # Outputs
///
/// None
///
/// # Usage
/// ```no_run
/// use fsdr_blocks::sigmf::SigMFSinkBuilder;
/// use futuresdr::runtime::Flowgraph;
///
/// let mut fg = Flowgraph::new();
///
/// let mut builder = SigMFSinkBuilder::from("my_filename");
/// let sink = builder.build::<u16>();
/// ```
#[cfg_attr(docsrs, doc(cfg(not(target_arch = "wasm32"))))]
pub struct SigMFSink<T, W, M>
where
    T: Send + 'static + Sized,
    W: Write,
    M: Write,
{
    pub writer: W,
    pub meta_writer: M,
    pub description: DescriptionBuilder,
    global_index: usize,
    sample_index: usize,
    _sample_type: std::marker::PhantomData<T>,
    _writer_type: std::marker::PhantomData<W>,
    _meta_writer_type: std::marker::PhantomData<M>,
}

impl<T, W, M> SigMFSink<T, W, M>
where
    T: Send + 'static + Sized + std::marker::Sync,
    W: Write + std::marker::Send + 'static, // + std::marker::Sync + std::marker::Send + std::marker::Unpin,
    M: Write + std::marker::Send + 'static, //std::io::Write, // + Send + std::marker::Sync,
{
    /// Create FileSink block
    pub fn new(writer: W, description: DescriptionBuilder, meta_writer: M) -> Block {
        Block::new(
            BlockMetaBuilder::new("SigMFSink").build(),
            StreamIoBuilder::new().add_input::<T>("in").build(),
            MessageIoBuilder::new().build(),
            SigMFSink::<T, W, M> {
                writer,
                meta_writer,
                description,
                global_index: 0,
                sample_index: 0,
                _sample_type: std::marker::PhantomData,
                _writer_type: std::marker::PhantomData,
                _meta_writer_type: std::marker::PhantomData,
            },
        )
    }
}

pub fn convert_annotation_to_pmt(annot: &Annotation) -> Pmt {
    let mut dict = HashMap::<String, Pmt>::new();
    if let Some(label) = &annot.label {
        dict.insert("label".to_string(), Pmt::String(label.clone()));
    }
    if let Some(annot_sample_start) = annot.sample_start {
        dict.insert("sample_start".to_string(), Pmt::Usize(annot_sample_start));
    }
    if let Some(annot_sample_count) = annot.sample_count {
        dict.insert("sample_count".to_string(), Pmt::Usize(annot_sample_count));
    }
    // TODO
    Pmt::MapStrPmt(dict)
}

#[doc(hidden)]
#[async_trait]
impl<T, W, M> Kernel for SigMFSink<T, W, M>
where
    T: Send + 'static + Sized + std::marker::Sync,
    W: Write + Send + 'static,
    M: Write + Send, //std::io::Write + Send + std::marker::Sync,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice_unchecked::<u8>();

        let item_size = std::mem::size_of::<T>();
        let items = i.len() / item_size;

        if items > 0 {
            let i = &i[..items * item_size];
            let _ = self.writer.write_all(i)?;
        }

        if sio.input(0).finished() {
            io.finished = true;
        }

        sio.input(0).consume(items);
        Ok(())
    }

    // async fn init(
    //     &mut self,
    //     _sio: &mut StreamIo,
    //     _mio: &mut MessageIo<Self>,
    //     _meta: &mut BlockMeta,
    // ) -> Result<()> {
    //     Ok(())
    // }

    async fn deinit(
        &mut self,
        _sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let desc = self.description.build()?;
        desc.to_writer_pretty(&mut self.meta_writer)?;
        Ok(())
    }
}

pub struct SigMFSinkBuilder {
    basename: PathBuf,
    datatype: DatasetFormat,
}

impl SigMFSinkBuilder {
    pub fn datatype(self, data: DatasetFormat) -> Self {
        SigMFSinkBuilder {
            basename: self.basename,
            datatype: data,
        }
    }
}

impl From<&PathBuf> for SigMFSinkBuilder {
    fn from(value: &PathBuf) -> Self {
        SigMFSinkBuilder {
            basename: value.to_path_buf(),
            datatype: DatasetFormat::Cf32Le,
        }
    }
}

impl From<PathBuf> for SigMFSinkBuilder {
    fn from(value: PathBuf) -> Self {
        SigMFSinkBuilder {
            basename: value.to_path_buf(),
            datatype: DatasetFormat::Cf32Le,
        }
    }
}

impl From<String> for SigMFSinkBuilder {
    fn from(value: String) -> Self {
        SigMFSinkBuilder {
            basename: PathBuf::from(value),
            datatype: DatasetFormat::Cf32Le,
        }
    }
}

impl From<&OsStr> for SigMFSinkBuilder {
    fn from(value: &OsStr) -> Self {
        SigMFSinkBuilder {
            basename: PathBuf::from(value),
            datatype: DatasetFormat::Cf32Le,
        }
    }
}

impl From<&str> for SigMFSinkBuilder {
    fn from(value: &str) -> Self {
        SigMFSinkBuilder {
            basename: PathBuf::from(value),
            datatype: DatasetFormat::Cf32Le,
        }
    }
}

impl SigMFSinkBuilder {
    pub async fn build<T: Sized + 'static + Sync + Send>(&mut self) -> Result<Block> {
        let desc = DescriptionBuilder::from(self.datatype);
        self.basename.set_extension("sigmf-data");
        let actual_file = std::fs::File::create(&self.basename)?;
        self.basename.set_extension("sigmf-meta");
        let meta_file = std::fs::File::create(&self.basename)?;
        Ok(SigMFSink::<T, _, _>::new(actual_file, desc, meta_file))
    }
}
