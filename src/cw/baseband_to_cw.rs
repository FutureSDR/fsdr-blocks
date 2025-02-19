use async_trait::async_trait;
use std::ops::RangeInclusive;

use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::Result;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::runtime::{Block, TypedBlock};

use crate::cw::shared::CWAlphabet::{self, *};

pub struct BaseBandToCW {
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

impl BaseBandToCW {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        accuracy: usize, // 100 = 100% accuracy = How accurate the timeslots for symbols and between symbols have to be kept
        samples_per_dot: usize,
    ) -> Block {
        Block::from_typed(Self::new_typed(accuracy, samples_per_dot))
    }

    pub fn new_typed(
        accuracy: usize, // 100 = 100% accuracy = How accurate the timeslots for symbols and between symbols have to be kept
        samples_per_dot: usize,
    ) -> TypedBlock<Self> {
        let tolerance_per_dot =
            (samples_per_dot as f32 - ((accuracy as f32 / 100.) * samples_per_dot as f32)) as usize;
        let dot_range = samples_per_dot - tolerance_per_dot..=samples_per_dot + tolerance_per_dot;
        let dash_range =
            3 * samples_per_dot - tolerance_per_dot..=3 * samples_per_dot + tolerance_per_dot;
        let letterspace_range =
            3 * samples_per_dot - tolerance_per_dot..=3 * samples_per_dot + tolerance_per_dot;
        let wordspace_range =
            7 * samples_per_dot - tolerance_per_dot..=7 * samples_per_dot + tolerance_per_dot;

        println!("samples per dot: {}", samples_per_dot);
        println!("dot_range: {:?}", dot_range);
        println!("dash_range: {:?}", dash_range);
        println!("letterspace_range: {:?}", letterspace_range);
        println!("wordspace_range: {:?}", wordspace_range);

        TypedBlock::new(
            BlockMetaBuilder::new("BBToCW").build(),
            StreamIoBuilder::new()
                .add_input::<f32>("in")
                .add_output::<CWAlphabet>("out")
                .build(),
            MessageIoBuilder::new().build(),
            BaseBandToCW {
                samples_per_dot,
                sample_count: 0,
                power_before: 0.,
                tolerance_per_dot, // // Tolerance towards the sending end in sticking to the time slots
                dot_range,         // How many samples are still interpreted as a dot
                dash_range,
                letterspace_range,
                wordspace_range,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for BaseBandToCW {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<f32>();
        let o = sio.output(0).slice::<CWAlphabet>();
        if o.is_empty() {
            return Ok(());
        }

        let mut consumed = 0;
        let mut produced = 0;
        let mut end_of_transmission = true;
        let threshold = 0.5; //(self.avg_power_min + self.avg_power_max) / 2.;

        let mut symbol = None;
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

                println!(
                    "Signal was paused for: {} -> {:?}",
                    self.sample_count,
                    symbol.or(None)
                );

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

                println!(
                    "Signal was present for: {} -> {:?}",
                    self.sample_count,
                    symbol.or(None)
                );

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
        }

        sio.input(0).consume(consumed);
        sio.output(0).produce(produced);

        if sio.input(0).finished() && consumed == i.len() {
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

    pub fn samples_per_dot(mut self, samles_per_dot: usize) -> BaseBandToCWBuilder {
        self.samles_per_dot = samles_per_dot;
        self
    }

    pub fn accuracy(mut self, accuracy: usize) -> BaseBandToCWBuilder {
        self.accuracy = accuracy;
        self
    }

    pub fn build(self) -> Block {
        BaseBandToCW::new(self.accuracy, self.samles_per_dot)
    }
}
