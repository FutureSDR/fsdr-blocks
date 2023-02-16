use futuresdr::anyhow::Result;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

pub struct Deinterleave<A>
where
    A: Send + 'static + Copy,
{
    _p1: std::marker::PhantomData<A>,
    first: bool,
}

impl<A> Deinterleave<A>
where
    A: Send + 'static + Copy,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Block {
        Block::new(
            BlockMetaBuilder::new("Deinterleave").build(),
            StreamIoBuilder::new()
                .add_input::<A>("in")
                .add_output::<A>("out0")
                .add_output::<A>("out1")
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            Deinterleave {
                _p1: std::marker::PhantomData,
                first: true,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<A> Kernel for Deinterleave<A>
where
    A: Send + 'static + Copy,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i0 = sio.input(0).slice::<A>();
        let mut m0 = 0;
        let mut m1 = 0;
        let o0 = sio.output(0).slice::<A>();
        let o1 = sio.output(1).slice::<A>();

        let mut it0 = o0.iter_mut();
        let mut it1 = o1.iter_mut();

        for x in i0.iter() {
            if self.first {
                let d = it0.next();
                if d.is_none() {
                    break;
                }
                let d = d.expect("");
                *d = *x;
                m0 += 1;
            } else {
                let d = it1.next();
                if d.is_none() {
                    break;
                }
                let d = d.expect("");
                *d = *x;
                m1 += 1;
            }
            self.first = !self.first;
        }

        let m = m0 + m1;
        sio.input(0).consume(m);
        sio.output(0).produce(m0);
        sio.output(1).produce(m1);

        if sio.input(0).finished() && m == i0.len() {
            io.finished = true;
        }

        Ok(())
    }
}
