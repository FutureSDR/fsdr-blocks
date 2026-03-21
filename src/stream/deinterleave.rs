use core::simd::Simd;
use futuresdr::prelude::*;

/// This blocks deinterleave a unique stream into two separate stream.
/// Typically used to deinterleave iq stream into of stream for `i` and one for `q`.
///
/// # Usage
/// ```
/// use fsdr_blocks::stream::Deinterleave;
/// let blk = Deinterleave::<f32>::new();
/// ```
#[derive(Block)]
pub struct Deinterleave<
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O0: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
    O1: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    #[input]
    input: I,
    #[output]
    out0: O0,
    #[output]
    out1: O1,
    first: bool,
}

impl<A, I, O0, O1> Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    pub fn new() -> Self {
        Self {
            input: I::default(),
            out0: O0::default(),
            out1: O1::default(),
            first: true,
        }
    }
}

pub trait DeinterleaveSupported: Copy {
    fn deinterleave(
        first: &mut bool,
        input: &[Self],
        out0: &mut [Self],
        out1: &mut [Self],
    ) -> (usize, usize, usize);
}

fn deinterleave_scalar_logic<A: Copy>(
    first: &mut bool,
    input: &[A],
    out0: &mut [A],
    out1: &mut [A],
) -> (usize, usize, usize) {
    let mut n_in = input.len();
    let n_o0 = out0.len();
    let n_o1 = out1.len();

    let mut i_ptr = 0;
    let mut o0_ptr = 0;
    let mut o1_ptr = 0;

    if !*first && n_in > 0 && n_o1 > 0 {
        out1[o1_ptr] = input[i_ptr];
        i_ptr += 1;
        o1_ptr += 1;
        n_in -= 1;
        *first = true;
    }

    let n = (n_in / 2).min(n_o0 - o0_ptr).min(n_o1 - o1_ptr);
    for j in 0..n {
        out0[o0_ptr + j] = input[i_ptr + 2 * j];
        out1[o1_ptr + j] = input[i_ptr + 2 * j + 1];
    }

    i_ptr += 2 * n;
    o0_ptr += n;
    o1_ptr += n;
    n_in -= 2 * n;

    if *first && n_in > 0 && out0.len() > o0_ptr {
        out0[o0_ptr] = input[i_ptr];
        i_ptr += 1;
        o0_ptr += 1;
        *first = false;
    }
    (i_ptr, o0_ptr, o1_ptr)
}

impl<A: Copy> DeinterleaveSupported for A {
    default fn deinterleave(
        first: &mut bool,
        input: &[Self],
        out0: &mut [Self],
        out1: &mut [Self],
    ) -> (usize, usize, usize) {
        deinterleave_scalar_logic(first, input, out0, out1)
    }
}

macro_rules! impl_deinterleave_simd {
    ($($t:ty),*) => {
        $(
            impl DeinterleaveSupported for $t {
                fn deinterleave(first: &mut bool, input: &[Self], out0: &mut [Self], out1: &mut [Self]) -> (usize, usize, usize) {
                    let mut n_in = input.len();
                    let n_o0 = out0.len();
                    let n_o1 = out1.len();

                    let mut i_ptr = 0;
                    let mut o0_ptr = 0;
                    let mut o1_ptr = 0;

                    if !*first && n_in > 0 && n_o1 > 0 {
                        out1[o1_ptr] = input[i_ptr];
                        i_ptr += 1;
                        o1_ptr += 1;
                        n_in -= 1;
                        *first = true;
                    }

                    const LANES: usize = 8;
                    let n_simd = (n_in / (2 * LANES)).min((n_o0 - o0_ptr) / LANES).min((n_o1 - o1_ptr) / LANES);

                    for _ in 0..n_simd {
                        let v0 = Simd::<$t, LANES>::from_slice(&input[i_ptr..i_ptr + LANES]);
                        let v1 = Simd::<$t, LANES>::from_slice(&input[i_ptr + LANES..i_ptr + 2 * LANES]);

                        let (even, odd) = v0.deinterleave(v1);
                        even.copy_to_slice(&mut out0[o0_ptr..o0_ptr + LANES]);
                        odd.copy_to_slice(&mut out1[o1_ptr..o1_ptr + LANES]);

                        i_ptr += 2 * LANES;
                        o0_ptr += LANES;
                        o1_ptr += LANES;
                    }

                    let (i_rem, o0_rem, o1_rem) = deinterleave_scalar_logic(first, &input[i_ptr..], &mut out0[o0_ptr..], &mut out1[o1_ptr..]);
                    (i_ptr + i_rem, o0_ptr + o0_rem, o1_ptr + o1_rem)
                }
            }
        )*
    };
}

impl_deinterleave_simd!(f32, u8, i8, i16);

impl<A, I, O0, O1> Default for Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<A, I, O0, O1> Kernel for Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy + DeinterleaveSupported,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let (m, m0, m1) = {
            let i = self.input.slice();
            let o0 = self.out0.slice();
            let o1 = self.out1.slice();

            A::deinterleave(&mut self.first, i, o0, o1)
        };

        self.input.consume(m);
        self.out0.produce(m0);
        self.out1.produce(m1);

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}
