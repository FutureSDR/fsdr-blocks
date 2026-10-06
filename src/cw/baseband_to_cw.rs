use std::ops::RangeInclusive;

use futuresdr::runtime::dev::prelude::*;

use crate::cw::shared::CWAlphabet::{self, *};

#[derive(Block)]
pub struct BaseBandToCW<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = CWAlphabet> = DefaultCpuWriter<CWAlphabet>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    samples_per_dot: usize,
    sample_count: usize,
    power_before: f32,
    tolerance_per_dot: usize,
    // Tolerance towards the sending end in sticking to the time slots
    dot_range: RangeInclusive<usize>,
    // How many samples are still interpreted as a dot
    dash_range: RangeInclusive<usize>,
    letterspace_range: RangeInclusive<usize>,
    wordspace_range: RangeInclusive<usize>,
}

impl<I, O> BaseBandToCW<I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = CWAlphabet>,
{
    pub fn new(
        accuracy: usize, // 100 = 100% accuracy = How accurate the timeslots for symbols and between symbols have to be kept
        samples_per_dot: usize,
    ) -> Self {
        let tolerance_per_dot = f32::algebraic_sub(
            samples_per_dot as f32,
            f32::algebraic_mul(
                f32::algebraic_div(accuracy as f32, 100.0),
                samples_per_dot as f32,
            ),
        ) as usize;
        let dot_range = samples_per_dot - tolerance_per_dot..=samples_per_dot + tolerance_per_dot;
        let dash_range =
            3 * samples_per_dot - tolerance_per_dot..=3 * samples_per_dot + tolerance_per_dot;
        let letterspace_range =
            3 * samples_per_dot - tolerance_per_dot..=3 * samples_per_dot + tolerance_per_dot;
        let wordspace_range =
            7 * samples_per_dot - tolerance_per_dot..=7 * samples_per_dot + tolerance_per_dot;

        // println!("samples per dot: {}", samples_per_dot);
        // println!("dot_range: {:?}", dot_range);
        // println!("dash_range: {:?}", dash_range);
        // println!("letterspace_range: {:?}", letterspace_range);
        // println!("wordspace_range: {:?}", wordspace_range);

        BaseBandToCW {
            input: I::default(),
            output: O::default(),
            samples_per_dot,
            sample_count: 0,
            power_before: 0.,
            tolerance_per_dot, // // Tolerance towards the sending end in sticking to the time slots
            dot_range,         // How many samples are still interpreted as a dot
            dash_range,
            letterspace_range,
            wordspace_range,
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for BaseBandToCW<I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = CWAlphabet>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (consumed, produced, finished) = {
            let i = self.input.slice();
            let o = self.output.slice();
            if o.len() < 2 {
                // We might produce 2 symbols at once (End of transmission)
                return Ok(());
            }

            let mut consumed = 0;
            let mut produced = 0;
            let mut end_of_transmission = true;
            let threshold = 0.5; //(self.avg_power_min + self.avg_power_max) / 2.;

            let mut symbol = None;
            let i_len = i.len();
            for sample in i.iter() {
                let power = (*sample).abs(); //.powi(2); // Not required

                if (power > threshold) && (self.power_before <= threshold) {
                    // Signal is starting
                    match self.sample_count {
                        x if self.wordspace_range.contains(&x) => {
                            symbol = Some(WordSpace);
                        } // Wordspace 7 dots (incl tolerance)
                        x if self.letterspace_range.contains(&x) => {
                            symbol = Some(LetterSpace);
                        } // Letterspace (Longer than 3 dots (incl tolerance), but shorter than 7 dots (incl tolerance))
                        x if self.dot_range.contains(&x) => {} // SymbolSpace (Is a valid symbol)
                        _ => {
                            //info!("Signal pause not a symbol: {} samples", self.sample_count);
                        }
                    }

                    // println!(
                    //     "Signal was paused for: {} -> {:?}",
                    //     self.sample_count,
                    //     symbol.or(None)
                    // );

                    self.sample_count = 0;
                    end_of_transmission = false;
                }
                if (power <= threshold) && (self.power_before > threshold) {
                    // Signal is stopping
                    match self.sample_count {
                        x if self.dot_range.contains(&x) => {
                            symbol = Some(Dot);
                        }
                        x if self.dash_range.contains(&x) => {
                            symbol = Some(Dash);
                        }
                        _ => {
                            //info!("Signal length not a symbol: {} samples", self.sample_count);
                        }
                    }

                    // println!(
                    //     "Signal was present for: {} -> {:?}",
                    //     self.sample_count,
                    //     symbol.or(None)
                    // );

                    self.sample_count = 0;
                }

                if let Some(val) = symbol {
                    o[produced] = val;
                    produced += 1;
                    symbol = None;
                }

                // Special Case: No signal has been received for a longer time than a wordspace needs.
                if self.sample_count > (self.tolerance_per_dot + (7 * self.samples_per_dot))
                    && !end_of_transmission
                {
                    // End of transmission
                    //println!("Transmission ended!");
                    end_of_transmission = true;
                    o[produced] = LetterSpace;
                    o[produced + 1] = WordSpace;
                    produced += 2;
                }

                if self.sample_count == usize::MAX {
                    // Dont overflow
                    self.sample_count = 0;
                }

                self.sample_count += 1;
                self.power_before = power;
                consumed += 1;

                if produced >= o.len() - 2 {
                    break;
                }
            }
            (
                consumed,
                produced,
                self.input.finished() && consumed == i_len,
            )
        };

        if consumed > 0 {
            self.input.consume(consumed);
        }
        if produced > 0 {
            self.output.produce(produced);
        }

        if finished {
            io.finished = true;
        }

        Ok(())
    }
}

pub struct BaseBandToCWBuilder {
    samles_per_dot: usize,
    accuracy: usize,
}

impl Default for BaseBandToCWBuilder {
    fn default() -> Self {
        BaseBandToCWBuilder {
            samles_per_dot: 60,
            accuracy: 90,
        }
    }
}

impl BaseBandToCWBuilder {
    pub fn new() -> BaseBandToCWBuilder {
        BaseBandToCWBuilder::default()
    }

    pub fn samples_per_dot(mut self, samples_per_dot: usize) -> BaseBandToCWBuilder {
        self.samles_per_dot = samples_per_dot;
        self
    }

    pub fn accuracy(mut self, accuracy: usize) -> BaseBandToCWBuilder {
        self.accuracy = accuracy;
        self
    }

    pub fn build(self) -> BaseBandToCW {
        BaseBandToCW::new(self.accuracy, self.samles_per_dot)
    }
}
