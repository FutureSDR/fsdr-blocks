use futuresdr::anyhow::Result;
use futuresdr::async_trait::async_trait;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::runtime::{Block, TypedBlock};

use crate::cw::shared::get_alphabet;
use crate::cw::shared::CWAlphabet::{self, LetterSpace, WordSpace};
use bimap::BiMap;

pub struct CWToChar {
    // Required to keep the state of already received pulses
    symbol_vec: Vec<CWAlphabet>,
    alphabet: BiMap<char, Vec<CWAlphabet>>,
}

impl CWToChar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(alphabet: BiMap<char, Vec<CWAlphabet>>) -> Block {
        Block::from_typed(Self::new_typed(alphabet))
    }

    pub fn new_typed(alphabet: BiMap<char, Vec<CWAlphabet>>) -> TypedBlock<Self> {
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
            },
        )
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
        // Not doing any checks on the output buffer length here.
        // Assuming, that i and o are of the same length.
        // Assuming, that one input sample generates at max one output sample.
        let i = sio.input(0).slice::<CWAlphabet>();
        let o = sio.output(0).slice::<char>();

        let mut produced = 0;

        self.symbol_vec.append(&mut i.to_vec());

        if self.symbol_vec.contains(&WordSpace) || self.symbol_vec.contains(&LetterSpace) {
            for (index, c) in self
                .symbol_vec
                .split_inclusive(|c| c == &LetterSpace || c == &WordSpace)
                .filter_map(|c| c.split_last())
                .map(|(last, elements)| {
                    //println!("last: {}, elements: {:?}", last, elements);
                    if last == &WordSpace {
                        *self.alphabet.get_by_right(&vec![WordSpace]).unwrap_or(&'_')
                    } else {
                        *self.alphabet.get_by_right(elements).unwrap_or(&'_')
                    }
                })
                .enumerate()
            {
                o[index] = c;
                produced = index + 1;
                //println!("c: {}, index: {}, produced: {}", c, index, produced);
            }

            self.symbol_vec.clear();
        }

        sio.input(0).consume(i.len());
        sio.output(0).produce(produced);

        if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
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
        CWToChar::new(self.alphabet)
    }
}
