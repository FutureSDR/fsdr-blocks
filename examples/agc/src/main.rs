use fsdr_blocks::agc::AgcBuilder;
use futuresdr::blocks::{Combine, SignalSourceBuilder, audio::AudioSink};
use futuresdr::prelude::*;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Generate 220Hz tone
    let src = SignalSourceBuilder::<f32>::sin(220.0, 48_000.0, 0.4, 0.0);
    // Modulation Wave for the 220Hz tone, changing from loud to silent every second
    let gain_change = SignalSourceBuilder::<f32>::sin(0.5, 48_000.0, 0.5, 0.0);
    // Modulate Tone with the modulation wave
    let combine = Combine::<_, f32, f32, f32>::new(|a: &f32, b: &f32| a * b);
    // Set the Automatic Gain Control settings
    let agc = AgcBuilder::<f32>::new()
        .squelch(0.0)
        .max_gain(65536.0)
        .adjustment_rate(0.1)
        .reference_power(1.0)
        .build();

    // Audiosink to output the modulated tone
    let audio_snk = AudioSink::new(48_000, 1)?;

    connect!(fg,
             src > in0.combine;
             gain_change > in1.combine;
             combine > agc > audio_snk;
    );
    let agc = agc.id();

    // Start the flowgraph and save the handle
    let rt = Runtime::new();
    let handle = rt.start(fg)?;

    // Keep changing gain and gain lock.
    loop {
        // Reference power of 1.0 is the power level we want to achieve
        println!("Setting reference power to 1.0");
        block_on(handle.call(agc, "reference_power", Pmt::F32(1.0)))?;

        // A high max gain allows to amplify a signal
        println!("Setting Max Gain to 65536.0");
        block_on(handle.call(agc, "max_gain", Pmt::F32(65536.0)))?;
        sleep(Duration::from_secs(5));

        // Setting a gain lock prevents gain changes from happening
        println!("Setting gain lock for 5s");
        block_on(handle.call(agc, "gain_lock", Pmt::Bool(true)))?;

        // Audio should get quiet faster, but gain is still locked here. It will be released after 5 seconds.
        println!("Setting reference power to 0.2");
        block_on(handle.call(agc, "reference_power", Pmt::F32(0.2)))?;
        sleep(Duration::from_secs(5));

        // Gain lock released! Audio should get more quiet here for 10 seconds
        println!("Releasing gain lock");
        block_on(handle.call(agc, "gain_lock", Pmt::Bool(false)))?;
        sleep(Duration::from_secs(10));
    }
}
