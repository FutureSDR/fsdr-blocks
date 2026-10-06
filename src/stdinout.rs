//! ## Blocks related to stdin/stdout serialization

use core::marker::PhantomData;
use futuresdr::num_complex::Complex32;
use std::io::Write;

#[derive(Clone, Copy)]
pub enum StdDirection {
    In,
    Out,
}

#[derive(Clone, Copy)]
pub enum BytesOrder {
    Native,
    BigEndian,
    LittleEndian,
}

/// Build blocks to serialize/deserialized stream from stdin/stdout.
/// It also takes care of endianness.
///
/// # Usage
///
/// Build a block that outputs a stream of u8 with native endianness to stdout:
/// ```
/// # use fsdr_blocks::stdinout::StdInOutBuilder;
/// let blk = StdInOutBuilder::<u8>::stdout().as_ne().build();
/// ```
///
/// Build a block that outputs a stream of `u8` with little endianness to stdout:
/// ```
/// # use fsdr_blocks::stdinout::StdInOutBuilder;
/// let blk = StdInOutBuilder::<u8>::stdout().as_le().build();
/// ```
pub struct StdInOutBuilder<A> {
    direction: StdDirection,
    marker_type: PhantomData<A>,
    bytes_order: BytesOrder,
}

impl<A> StdInOutBuilder<A> {
    pub fn stdin() -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            marker_type: PhantomData,
            direction: StdDirection::In,
            bytes_order: BytesOrder::Native,
        }
    }

    pub fn stdout() -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            marker_type: PhantomData,
            direction: StdDirection::Out,
            bytes_order: BytesOrder::Native,
        }
    }

    pub fn as_ne(self) -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            bytes_order: BytesOrder::Native,
            ..self
        }
    }

    pub fn as_le(self) -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            bytes_order: BytesOrder::LittleEndian,
            ..self
        }
    }

    pub fn as_be(self) -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            bytes_order: BytesOrder::BigEndian,
            ..self
        }
    }
}

pub trait ToEndianBytes {
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()>;
}

impl ToEndianBytes for u8 {
    #[inline(always)]
    fn write_to<W: Write>(&self, w: &mut W, _order: BytesOrder) -> std::io::Result<()> {
        w.write_all(&[*self])
    }
}

impl ToEndianBytes for i16 {
    #[inline(always)]
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()> {
        match order {
            BytesOrder::Native => w.write_all(&self.to_ne_bytes()),
            BytesOrder::LittleEndian => w.write_all(&self.to_le_bytes()),
            BytesOrder::BigEndian => w.write_all(&self.to_be_bytes()),
        }
    }
}

impl ToEndianBytes for f32 {
    #[inline(always)]
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()> {
        match order {
            BytesOrder::Native => w.write_all(&self.to_ne_bytes()),
            BytesOrder::LittleEndian => w.write_all(&self.to_le_bytes()),
            BytesOrder::BigEndian => w.write_all(&self.to_be_bytes()),
        }
    }
}

impl ToEndianBytes for Complex32 {
    #[inline(always)]
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()> {
        match order {
            BytesOrder::Native => {
                let bytes: [u8; 8] = unsafe { std::mem::transmute(*self) };
                w.write_all(&bytes)
            }
            BytesOrder::LittleEndian => {
                let mut bytes = [0u8; 8];
                bytes[..4].copy_from_slice(&self.re.to_le_bytes());
                bytes[4..].copy_from_slice(&self.im.to_le_bytes());
                w.write_all(&bytes)
            }
            BytesOrder::BigEndian => {
                let mut bytes = [0u8; 8];
                bytes[..4].copy_from_slice(&self.re.to_be_bytes());
                bytes[4..].copy_from_slice(&self.im.to_be_bytes());
                w.write_all(&bytes)
            }
        }
    }
}

use futuresdr::runtime::dev::prelude::*;

#[derive(Block)]
pub struct StdOutSink<
    A: ToEndianBytes + Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
> {
    #[input]
    input: I,
    bytes_order: BytesOrder,
}

impl<
    A: ToEndianBytes + Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
> StdOutSink<A, I>
{
    pub fn new(bytes_order: BytesOrder) -> Self {
        Self {
            input: I::default(),
            bytes_order,
        }
    }
}

#[doc(hidden)]
impl<
    A: ToEndianBytes + Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
> Kernel for StdOutSink<A, I>
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let m = i.len();
        if m > 0 {
            let mut stdout = std::io::BufWriter::new(std::io::stdout());
            match self.bytes_order {
                BytesOrder::Native => {
                    let bytes = unsafe {
                        std::slice::from_raw_parts(
                            i.as_ptr() as *const u8,
                            std::mem::size_of_val(i),
                        )
                    };
                    if let Err(e) = stdout.write_all(bytes)
                        && e.kind() != std::io::ErrorKind::BrokenPipe
                    {
                        eprintln!("StdInOut: write error: {e}");
                    }
                }
                BytesOrder::LittleEndian | BytesOrder::BigEndian => {
                    for sample in i {
                        if let Err(e) = sample.write_to(&mut stdout, self.bytes_order)
                            && e.kind() != std::io::ErrorKind::BrokenPipe
                        {
                            eprintln!("StdInOut: write error: {e}");
                        }
                    }
                }
            }
            self.input.consume(m);
        }

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}

impl<A: ToEndianBytes + Send + Sync + Default + Copy + std::fmt::Debug + 'static>
    StdInOutBuilder<A>
{
    pub fn build(self) -> StdOutSink<A> {
        match self.direction {
            StdDirection::Out => StdOutSink::new(self.bytes_order),
            StdDirection::In => todo!("stdin not yet implemented"),
        }
    }
}
