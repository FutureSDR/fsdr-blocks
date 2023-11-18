use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use futuresdr::anyhow::{anyhow, Result};
use futuresdr::futures::AsyncRead;
use futuresdr::futures::AsyncReadExt;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::runtime::{Block, Pmt, Tag};

use sigmf::RecordingBuilder;
use sigmf::{Annotation, Capture, Description};

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
/// let builder = SigMFSourceBuilder::from("my_filename");
/// let source = builder.build::<u16>();
/// ```
#[cfg_attr(docsrs, doc(cfg(not(target_arch = "wasm32"))))]
pub struct SigMFSource<T, R>
where
    T: Send + 'static + Sized,
    R: AsyncRead,
{
    reader: R,
    annotations: Vec<Annotation>,
    captures: Vec<Capture>,
    global_index: usize,
    sample_index: usize,
    _sample_type: std::marker::PhantomData<T>,
    _reader_type: std::marker::PhantomData<R>,
}

impl<'a, T, R> SigMFSource<T, R>
where
    T: Send + 'static + Sized + std::marker::Sync,
    R: AsyncRead + std::marker::Sync + std::marker::Send + std::marker::Unpin + 'static,
{
    /// Create FileSource block
    pub fn new(reader: R, desc: Description) -> Block {
        let annotations = if let Some(annot) = desc.annotations {
            annot
        } else {
            vec![]
        };
        let captures = if let Some(capts) = desc.captures {
            capts
        } else {
            vec![]
        };
        Block::new(
            BlockMetaBuilder::new("FileSource").build(),
            StreamIoBuilder::new().add_output::<T>("out").build(),
            MessageIoBuilder::new().build(),
            SigMFSource::<T, R> {
                reader,
                annotations,
                captures,
                global_index: 0,
                sample_index: 0,
                _sample_type: std::marker::PhantomData,
                _reader_type: std::marker::PhantomData,
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
impl<'a, T, R> Kernel for SigMFSource<T, R>
where
    T: Send + 'static + Sized + std::marker::Sync,
    R: AsyncRead + std::marker::Send + std::marker::Sync + std::marker::Unpin,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let out = sio.output(0).slice_unchecked::<u8>();
        let item_size = std::mem::size_of::<T>();

        debug_assert_eq!(out.len() % item_size, 0);

        let mut i = 0;

        while i < out.len() {
            match self.reader.read(&mut out[i..]).await {
                Ok(0) => {
                    io.finished = true;
                    break;
                }
                Ok(written) => {
                    i += written;
                }
                Err(e) => panic!("SigMFSource: Error reading data: {e:?}"),
            }
        }
        let amount = i / item_size;
        sio.output(0).produce(amount);
        while let Some(annot) = self.annotations.get(0) {
            if let Some(annot_sample_start) = annot.sample_start {
                let upper_sample_index = self.sample_index + amount;
                if (self.sample_index..upper_sample_index).contains(&annot_sample_start) {
                    let tag = convert_annotation_to_pmt(annot);
                    let tag = Tag::Data(tag);
                    sio.output(0)
                        .add_tag(annot_sample_start - self.sample_index, tag);

                    self.annotations.remove(0);
                } else {
                    break;
                }
            } else {
                // Skip all annotations without sample_start
                self.annotations.remove(0);
            }
        }
        self.sample_index += amount;

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
}

// impl Into<HashMap<String, Pmt>> for Annotation {
//     fn into(self) -> HashMap<String, Pmt> {
//         todo!()
//     }
// }

pub struct SigMFSourceBuilder {
    basename: PathBuf,
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
    pub async fn build<T: Sized + 'static + Send>(&self) -> Result<Block> {
        let mut record = RecordingBuilder::from(&self.basename);
        let (_, desc) = record.load_description()?;
        let global = desc.global()?;
        let datatype = global.datatype()?;

        // if datatype.is_real() {
        //     return Err(anyhow!("Expected complex dataset format, got {}", datatype))
        // }

        let actual_file = async_fs::File::open(&self.basename).await?;

        // #[cfg(target_endian = "little")]
        // {
        //     use sigmf::DatasetFormat::*;
        //     let filter = match datatype {
        //         &Cf32Le => ,
        //         _ => actual_file,
        //     }
        // }

        Ok(SigMFSource::<u32, _>::new(actual_file, desc))
    }
}
