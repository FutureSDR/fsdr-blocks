use futuresdr::anyhow::Result;
use futuresdr::async_trait::async_trait;
use futuresdr::runtime::{Block, TypedBlock};
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

use bimap::BiMap;
use futuresdr::log::info;
use crate::cw::shared::CWAlphabet::{self, LetterSpace, WordSpace};
use crate::cw::shared::get_alphabet;

pub struct CWToChar {
    symbol_vec: Vec<CWAlphabet>,
    // Required to keep the state of already received pulses
    alphabet: BiMap<char, Vec<CWAlphabet>>,
    workfn: usize,
}

impl CWToChar {
    pub fn new(alphabet: BiMap<char, Vec<CWAlphabet>>) -> Block {
        Block::from_typed(Self::new_typed(alphabet, 1))
    }

    pub fn new_typed(alphabet: BiMap<char, Vec<CWAlphabet>>, workfn: usize) -> TypedBlock<Self> {
        TypedBlock::new(
            BlockMetaBuilder::new("CWToChar").build(),
            StreamIoBuilder::new()
                .add_input::<CWAlphabet>("in")
                .add_output::<char>("out")
                .build(),
            MessageIoBuilder::new().build(),
            CWToChar {
                symbol_vec: vec![],
                alphabet,
                workfn
            },
        )
    }

    async fn work1(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        // Not doing any checks on the output buffer length here.
        // Assuming, that i and o are of the same length.
        // Assuming, that one input sample generates at max one output sample.
        let i = sio.input(0).slice::<CWAlphabet>();
        let o = sio.output(0).slice::<char>();

        let mut produced = 0;

        // Variant 1
        for (index, c) in i.split_inclusive(|c| c == &LetterSpace || c == &WordSpace)
            .filter_map(|c| c.split_last())
            .map(|(last, elements)| if last == &WordSpace {
                *self.alphabet.get_by_right(&vec![WordSpace]).unwrap_or(&'_')
            } else {
                *self.alphabet.get_by_right(elements).unwrap_or(&'_')
            })
            .enumerate() {
            o[index] = c;
            produced = index + 1;
        }

        sio.input(0).consume(i.len());
        sio.output(0).produce(produced);

        if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
    }

    async fn work2(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        // Not doing any checks on the output buffer length here.
        // Assuming, that i and o are of the same length.
        // Assuming, that one input sample generates at max one output sample.
        let i = sio.input(0).slice::<CWAlphabet>();
        let o = sio.output(0).slice::<char>();

        let mut consumed = 0;
        let mut produced = 0;

        // Variant 2
        for v in i.iter() {
            info!(": {}", v);
            match v {
                CWAlphabet::Dot | CWAlphabet::Dash => { self.symbol_vec.push(*v); }
                LetterSpace => {
                    if let Some(character) = self.alphabet.get_by_right(&self.symbol_vec) {
                        o[produced] = *character;
                        produced += 1;
                    }
                    self.symbol_vec.clear();
                }
                WordSpace => {
                    if let Some(character) = self.alphabet.get_by_right(&self.symbol_vec) {
                        o[produced] = *character;
                        produced += 1;
                    }
                    self.symbol_vec.clear();

                    self.symbol_vec.push(*v);
                    if let Some(character) = self.alphabet.get_by_right(&self.symbol_vec) {
                        o[produced] = *character;
                        produced += 1;
                    }
                    self.symbol_vec.clear();
                }
                CWAlphabet::Unknown => {}
            }
            consumed += 1;
        }


        sio.input(0).consume(consumed);
        sio.output(0).produce(produced);

        if sio.input(0).finished() && consumed == i.len() {
            io.finished = true;
        }

        Ok(())
    }

    async fn work3(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        // Not doing any checks on the output buffer length here.
        // Assuming, that i and o are of the same length.
        // Assuming, that one input sample generates at max one output sample.
        let i = sio.input(0).slice::<CWAlphabet>();
        let o = sio.output(0).slice::<char>();

        let mut consumed = 0;
        let mut produced = 0;

        // Variant 3
        for v in i.iter() {
            if (*v != LetterSpace) && (*v != WordSpace) {
                self.symbol_vec.push(*v);
                //info!("{:?}: ", self.symbol_vec);
            } else {
                info!("{:?}", self.symbol_vec);
                if let Some(character) = self.alphabet.get_by_right(&self.symbol_vec) {
                    //info!("{:?}: {}", self.symbol_vec, character);
                    o[produced] = *character;
                    produced += 1;
                }
                self.symbol_vec.clear();

                if *v == WordSpace { // Special case if sequence of pulse codes is not followed by a LetterSpace but a WordSpace
                    self.symbol_vec.push(*v);
                    if let Some(character) = self.alphabet.get_by_right(&self.symbol_vec) {
                        o[produced] = *character;
                        produced += 1;
                    }
                    self.symbol_vec.clear();
                }
            }
            consumed += 1;
        }


        sio.input(0).consume(consumed);
        sio.output(0).produce(produced);

        if sio.input(0).finished() && consumed == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for CWToChar {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        match self.workfn {
            2 => { self.work2(io, sio, _mio, _meta).await }
            3 => { self.work3(io, sio, _mio, _meta).await }
            _ => { self.work1(io, sio, _mio, _meta).await }
        }
    }
}


pub struct CWToCharBuilder {
    alphabet: BiMap<char, Vec<CWAlphabet>>,
}

impl Default for CWToCharBuilder {
    fn default() -> Self {
        CWToCharBuilder {
            alphabet: get_alphabet(),
        }
    }
}

impl CWToCharBuilder {
    pub fn new() -> CWToCharBuilder {
        CWToCharBuilder::default()
    }

    /*pub fn alphabet(mut self, alphabet: BiMap<char, Vec<CWAlphabet>>) -> CWToCharBuilder {
        self.alphabet = alphabet;
        self
    }*/

    pub fn build(self) -> Block {
        CWToChar::new(
            self.alphabet,
        )
    }
}
