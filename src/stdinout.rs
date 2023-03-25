//! ## Blocks related to stdin/stdout serialization

use core::marker::PhantomData;
use futuresdr::blocks::Sink;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use std::io::Write;

enum StdDirection {
    In,
    Out,
}

enum BytesOrder {
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

impl StdInOutBuilder<u8> {
    pub fn build(self) -> Block {
        match self.direction {
            StdDirection::Out => {
                let mut stdout = std::io::stdout();
                Sink::new(move |f: &i16| {
                    stdout
                        .write_all(&f.to_ne_bytes())
                        .expect("cannot write to stdout");
                    stdout.flush().expect("flush error on stdout");
                })
            }
            _ => todo!("stdin not yet implemented"),
        }
    }
}

impl StdInOutBuilder<i16> {
    pub fn build(self) -> Block {
        match self.direction {
            StdDirection::Out => {
                let mut stdout = std::io::stdout();
                match self.bytes_order {
                    BytesOrder::Native => Sink::new(move |f: &i16| {
                        stdout
                            .write_all(&f.to_ne_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                    BytesOrder::LittleEndian => Sink::new(move |f: &i16| {
                        stdout
                            .write_all(&f.to_le_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                    BytesOrder::BigEndian => Sink::new(move |f: &i16| {
                        stdout
                            .write_all(&f.to_be_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                }
            }
            _ => todo!("stdin not yet implemented"),
        }
    }
}

impl StdInOutBuilder<f32> {
    pub fn build(self) -> Block {
        match self.direction {
            StdDirection::Out => {
                let mut stdout = std::io::stdout();
                match self.bytes_order {
                    BytesOrder::Native => Sink::new(move |f: &f32| {
                        stdout
                            .write_all(&f.to_ne_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                    BytesOrder::LittleEndian => Sink::new(move |f: &f32| {
                        stdout
                            .write_all(&f.to_le_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                    BytesOrder::BigEndian => Sink::new(move |f: &f32| {
                        stdout
                            .write_all(&f.to_be_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                }
            }
            _ => todo!("stdin not yet implemented"),
        }
    }
}

impl StdInOutBuilder<Complex32> {
    pub fn build(self) -> Block {
        match self.direction {
            StdDirection::Out => {
                let mut stdout = std::io::stdout();
                match self.bytes_order {
                    BytesOrder::Native => Sink::new(move |f: &Complex32| {
                        stdout
                            .write_all(&f.re.to_ne_bytes())
                            .expect("cannot write to stdout");
                        stdout
                            .write_all(&f.im.to_ne_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                    BytesOrder::LittleEndian => Sink::new(move |f: &Complex32| {
                        stdout
                            .write_all(&f.re.to_le_bytes())
                            .expect("cannot write to stdout");
                        stdout
                            .write_all(&f.im.to_le_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                    BytesOrder::BigEndian => Sink::new(move |f: &Complex32| {
                        stdout
                            .write_all(&f.re.to_be_bytes())
                            .expect("cannot write to stdout");
                        stdout
                            .write_all(&f.im.to_be_bytes())
                            .expect("cannot write to stdout");
                        stdout.flush().expect("flush error on stdout");
                    }),
                }
            }
            _ => todo!("stdin not yet implemented"),
        }
    }
}
