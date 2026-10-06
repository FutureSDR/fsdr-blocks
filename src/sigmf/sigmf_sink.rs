use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;

use futuresdr::runtime::dev::prelude::*;

use sigmf::Annotation;
use sigmf::{DatasetFormat, DescriptionBuilder};

use crate::serde_pmt::from_pmt;

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
#[derive(Block)]
pub struct SigMFSink<
    T: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    W: Write + Send + 'static,
    M: Write + Send + 'static,
    I: CpuBufferReader<Item = T> = DefaultCpuReader<T>,
> {
    #[input]
    input: I,
    pub writer: W,
    pub meta_writer: M,
    pub description: DescriptionBuilder,
    // global_index: usize,
    // sample_index: usize,
}

impl<T, W, M, I> SigMFSink<T, W, M, I>
where
    T: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    W: Write + Send + 'static,
    M: Write + Send + 'static,
    I: CpuBufferReader<Item = T>,
{
    /// Create FileSink block
    pub fn new(writer: W, description: DescriptionBuilder, meta_writer: M) -> Self {
        SigMFSink {
            input: I::default(),
            writer,
            meta_writer,
            description,
        }
    }
}

pub fn convert_pmt_to_annotation(value: &Pmt) -> Option<Annotation> {
    let annot: crate::serde_pmt::error::Result<Annotation> = from_pmt(value.clone());
    annot.ok()
}

#[doc(hidden)]
impl<T, W, M, I> Kernel for SigMFSink<T, W, M, I>
where
    T: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    W: Write + Send + 'static,
    M: Write + Send + 'static,
    I: CpuBufferReader<Item = T>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let items = {
            let (i, tags) = self.input.slice_with_tags();
            let items = i.len();

            if items > 0 {
                // Safety: casts contiguous slice of plain data items to byte slice for I/O writing
                let bytes = unsafe {
                    std::slice::from_raw_parts(i.as_ptr() as *const u8, std::mem::size_of_val(i))
                };
                self.writer.write_all(bytes)?;
            }
            for item in tags {
                if let Tag::Data(pmt) = &item.tag
                    && let Some(annot) = convert_pmt_to_annotation(pmt)
                {
                    self.description.add_annotation(annot)?;
                }
            }

            if self.input.finished() {
                io.finished = true;
            }
            items
        };

        if items > 0 {
            self.input.consume(items);
        }
        Ok(())
    }

    async fn deinit(&mut self, _mio: &mut MessageOutputs, _meta: &BlockMeta) -> Result<()> {
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
    pub async fn build<T: Send + Sync + Default + Copy + std::fmt::Debug + 'static>(
        &mut self,
    ) -> Result<SigMFSink<T, std::fs::File, std::fs::File>> {
        let desc = DescriptionBuilder::from(self.datatype);
        self.basename.set_extension("sigmf-data");
        let actual_file = std::fs::File::create(&self.basename)?;
        self.basename.set_extension("sigmf-meta");
        let meta_file = std::fs::File::create(&self.basename)?;
        Ok(SigMFSink::<T, _, _>::new(actual_file, desc, meta_file))
    }
}
