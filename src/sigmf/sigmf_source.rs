use std::ffi::OsStr;
use std::path::PathBuf;

use futuresdr::futures::AsyncRead;
use futuresdr::futures::AsyncReadExt;
use futuresdr::runtime::dev::prelude::*;

use sigmf::RecordingBuilder;
use sigmf::{Annotation, Description};

use super::BytesConveter;
use crate::serde_pmt;

/// Read samples from a SigMF file.
///
/// # Inputs
///
/// No inputs.
///
/// # Outputs
///
/// `out`: Output samples
///
/// # Usage
/// ```no_run
/// use fsdr_blocks::sigmf::SigMFSourceBuilder;
/// use futuresdr::runtime::Flowgraph;
///
/// let mut fg = Flowgraph::new();
///
/// // Loads samples as unsigned 16-bits integer from the file `my_filename.sigmf-data` with
/// // conversion applied depending on the data type actually described in `my_filename.sigmf-meta`
/// let mut builder = SigMFSourceBuilder::from("my_filename");
/// let source = builder.build::<u16>();
/// ```
use std::collections::VecDeque;

#[cfg_attr(docsrs, doc(cfg(not(target_arch = "wasm32"))))]
#[derive(Block)]
pub struct SigMFSource<
    T: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    R: AsyncRead + Send + Sync + Unpin + 'static,
    F: FnMut(&[u8]) -> T + Send + 'static,
    O: CpuBufferWriter<Item = T> = DefaultCpuWriter<T>,
> {
    #[output]
    output: O,
    reader: R,
    annotations: VecDeque<Annotation>,
    sample_index: usize,
    converter: F,
    item_size: usize,
    buf: Vec<u8>,
}

impl<T, R, F, O> SigMFSource<T, R, F, O>
where
    T: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    R: AsyncRead + Send + Sync + Unpin + 'static,
    F: FnMut(&[u8]) -> T + Send + 'static,
    O: CpuBufferWriter<Item = T>,
{
    /// Create FileSource block
    pub fn new(reader: R, desc: Description, converter: F) -> Result<Self> {
        let global = desc.global()?;
        let datatype = *global.datatype()?;
        let annotations = desc.annotations.unwrap_or_default().into();
        Ok(SigMFSource {
            output: O::default(),
            reader,
            annotations,
            sample_index: 0,
            converter,
            item_size: datatype.size(),
            buf: Vec::new(),
        })
    }
}

#[doc(hidden)]
impl<T, R, F, O> Kernel for SigMFSource<T, R, F, O>
where
    T: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    R: AsyncRead + Send + Sync + Unpin + 'static,
    F: FnMut(&[u8]) -> T + Send + 'static,
    O: CpuBufferWriter<Item = T>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let n_read_items = {
            let (o, mut o_tags) = self.output.slice_with_tags();
            let needed_bytes = o.len() * self.item_size;
            self.buf.resize(needed_bytes, 0);

            let mut n_read_items = 0;
            match self.reader.read(&mut self.buf).await {
                Ok(0) => {
                    io.finished = true;
                }
                Ok(n_read_bytes) => {
                    n_read_items = n_read_bytes / self.item_size;
                    for (v, r) in self.buf[..n_read_bytes]
                        .chunks_exact(self.item_size)
                        .zip(o.iter_mut())
                    {
                        *r = (self.converter)(v);
                    }
                }
                Err(e) => panic!("SigMFSource: Error reading data: {e:?}"),
            }

            while let Some(annot) = self.annotations.front() {
                if let Some(annot_sample_start) = annot.sample_start {
                    let upper_sample_index = self.sample_index + n_read_items;
                    if (self.sample_index..upper_sample_index).contains(&annot_sample_start) {
                        let annot = self.annotations.pop_front().unwrap();
                        let tag_index = annot_sample_start - self.sample_index;
                        let pmt = serde_pmt::to_pmt(&annot)?;
                        o_tags.add_tag(tag_index, Tag::Data(pmt));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            self.sample_index += n_read_items;
            n_read_items
        };

        if n_read_items > 0 {
            self.output.produce(n_read_items);
        }
        Ok(())
    }
}

pub struct SigMFSourceBuilder {
    basename: PathBuf,
}

pub struct SigMFSourceBuilderFromReader<R> {
    data: R,
    desc: Description,
}

impl From<&PathBuf> for SigMFSourceBuilder {
    fn from(value: &PathBuf) -> Self {
        SigMFSourceBuilder {
            basename: value.to_path_buf(),
        }
    }
}

impl From<PathBuf> for SigMFSourceBuilder {
    fn from(value: PathBuf) -> Self {
        SigMFSourceBuilder {
            basename: value.to_path_buf(),
        }
    }
}

impl From<String> for SigMFSourceBuilder {
    fn from(value: String) -> Self {
        SigMFSourceBuilder {
            basename: PathBuf::from(value),
        }
    }
}

impl From<&OsStr> for SigMFSourceBuilder {
    fn from(value: &OsStr) -> Self {
        SigMFSourceBuilder {
            basename: PathBuf::from(value),
        }
    }
}

impl From<&str> for SigMFSourceBuilder {
    fn from(value: &str) -> Self {
        SigMFSourceBuilder {
            basename: PathBuf::from(value),
        }
    }
}

impl SigMFSourceBuilder {
    pub fn with_data_and_description<R: AsyncRead>(
        reader: R,
        desc: Description,
    ) -> SigMFSourceBuilderFromReader<R> {
        SigMFSourceBuilderFromReader { data: reader, desc }
    }

    pub async fn build<T: Send + Sync + Default + Copy + std::fmt::Debug + 'static>(
        &mut self,
    ) -> Result<SigMFSource<T, async_fs::File, impl FnMut(&[u8]) -> T + Send + 'static>>
    where
        sigmf::DatasetFormat: BytesConveter<T>,
    {
        let mut record = RecordingBuilder::from(&self.basename);
        let (_, desc) = record.load_description()?;
        let datatype = desc.global()?.datatype()?.to_owned();
        self.basename.set_extension("sigmf-data");
        let actual_file = async_fs::File::open(&self.basename).await?;
        SigMFSource::<T, _, _>::new(actual_file, desc, move |bytes| datatype.convert(bytes))
    }
}

impl<R> SigMFSourceBuilderFromReader<R>
where
    R: AsyncRead + Send + Sync + Unpin + 'static,
{
    pub async fn build<T: Send + Sync + Default + Copy + std::fmt::Debug + 'static>(
        self,
    ) -> Result<SigMFSource<T, R, impl FnMut(&[u8]) -> T + Send + 'static>>
    where
        sigmf::DatasetFormat: BytesConveter<T>,
    {
        let datatype = *self.desc.global()?.datatype()?;
        SigMFSource::<T, R, _>::new(self.data, self.desc, move |bytes| datatype.convert(bytes))
    }
}
