use futuresdr::runtime::dev::prelude::*;

use crate::cw::shared::CWAlphabet;
use crate::cw::shared::get_alphabet;
use bimap::BiMap;

#[derive(Block)]
pub struct CWToChar<
    I: CpuBufferReader<Item = CWAlphabet> = DefaultCpuReader<CWAlphabet>,
    O: CpuBufferWriter<Item = u32> = DefaultCpuWriter<u32>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    // Required to keep the state of already received pulses
    symbol_vec: Vec<CWAlphabet>,
    alphabet: BiMap<char, Vec<CWAlphabet>>,
}

impl<I, O> CWToChar<I, O>
where
    I: CpuBufferReader<Item = CWAlphabet>,
    O: CpuBufferWriter<Item = u32>,
{
    pub fn new(alphabet: BiMap<char, Vec<CWAlphabet>>) -> Self {
        CWToChar {
            input: I::default(),
            output: O::default(),
            symbol_vec: vec![],
            alphabet,
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for CWToChar<I, O>
where
    I: CpuBufferReader<Item = CWAlphabet>,
    O: CpuBufferWriter<Item = u32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i_len = {
            let i = self.input.slice();
            if !i.is_empty() {
                self.symbol_vec.extend_from_slice(i);
            }
            i.len()
        };
        if i_len > 0 {
            self.input.consume(i_len);
        }
        let o = self.output.slice();

        let mut produced = 0;
        if !self.symbol_vec.is_empty() && !o.is_empty() {
            let mut consumed_raw = 0;
            let mut current_token = Vec::new();
            let mut out_idx = 0;

            let input_finished = self.input.finished();

            for (idx, &sym) in self.symbol_vec.iter().enumerate() {
                if out_idx >= o.len() {
                    break;
                }
                match sym {
                    CWAlphabet::Dot | CWAlphabet::Dash => {
                        current_token.push(sym);
                    }
                    CWAlphabet::LetterSpace => {
                        if !current_token.is_empty() {
                            let ch = *self.alphabet.get_by_right(&current_token).unwrap_or(&'_');
                            o[out_idx] = ch as u32;
                            out_idx += 1;
                            current_token.clear();
                        }
                        consumed_raw = idx + 1;
                    }
                    CWAlphabet::WordSpace => {
                        if !current_token.is_empty() {
                            let ch = *self.alphabet.get_by_right(&current_token).unwrap_or(&'_');
                            o[out_idx] = ch as u32;
                            out_idx += 1;
                            current_token.clear();
                        }
                        if out_idx < o.len() {
                            let space = *self
                                .alphabet
                                .get_by_right(&vec![CWAlphabet::WordSpace])
                                .unwrap_or(&' ');
                            o[out_idx] = space as u32;
                            out_idx += 1;
                            consumed_raw = idx + 1;
                        }
                    }
                    CWAlphabet::Unknown => {
                        if !current_token.is_empty() {
                            let ch = *self.alphabet.get_by_right(&current_token).unwrap_or(&'_');
                            o[out_idx] = ch as u32;
                            out_idx += 1;
                            current_token.clear();
                        }
                        if out_idx < o.len() {
                            o[out_idx] = '_' as u32;
                            out_idx += 1;
                            consumed_raw = idx + 1;
                        }
                    }
                }
            }

            if input_finished && !current_token.is_empty() && out_idx < o.len() {
                let ch = *self.alphabet.get_by_right(&current_token).unwrap_or(&'_');
                o[out_idx] = ch as u32;
                out_idx += 1;
                consumed_raw = self.symbol_vec.len();
            }

            if consumed_raw > 0 {
                self.symbol_vec.drain(..consumed_raw);
            }
            produced = out_idx;
        }

        if produced > 0 {
            self.output.produce(produced);
        }
        if self.input.finished() && self.symbol_vec.is_empty() {
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

    pub fn build(self) -> CWToChar {
        CWToChar::new(self.alphabet)
    }
}
