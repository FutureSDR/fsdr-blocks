use futuresdr::anyhow::Result;
use futuresdr::blocks::signal_source::NCO;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

pub struct FrequencyShifter<A>
where
    A: Send + 'static + Copy,
{
    _p1: std::marker::PhantomData<A>,
    nco: NCO,
}

impl<A> FrequencyShifter<A>
where
    A: Send + 'static + Copy,
    FrequencyShifter<A>: futuresdr::runtime::Kernel,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(frequency: f32, sample_rate: f32) -> Block {
        let nco = NCO::new(
            0.0f32,
            2.0 * core::f32::consts::PI * frequency / sample_rate,
        );
        Block::new(
            BlockMetaBuilder::new("FrequencyShift").build(),
            StreamIoBuilder::new()
                .add_input::<A>("in")
                .add_output::<A>("out")
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            FrequencyShifter {
                _p1: std::marker::PhantomData,
                nco,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for FrequencyShifter<f32> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<f32>();
        let o = sio.output(0).slice::<f32>();

        let m = std::cmp::min(i.len(), o.len());
        if m > 0 {
            for (v, r) in i.iter().zip(o.iter_mut()) {
                *r = (*v) * self.nco.phase.cos();
                self.nco.step();
            }

            sio.input(0).consume(m);
            sio.output(0).produce(m);
        }

        if sio.input(0).finished() && m == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for FrequencyShifter<Complex32> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<Complex32>();
        let o = sio.output(0).slice::<Complex32>();

        let m = std::cmp::min(i.len(), o.len());
        if m > 0 {
            for (v, r) in i.iter().zip(o.iter_mut()) {
                *r = (*v) * Complex32::new(self.nco.phase.cos(), self.nco.phase.sin());
                self.nco.step();
            }

            sio.input(0).consume(m);
            sio.output(0).produce(m);
        }

        if sio.input(0).finished() && m == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}
