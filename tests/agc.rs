use fsdr_blocks::agc::AgcBuilder;
use futuresdr::blocks::{VectorSink, VectorSource};
use futuresdr::num_complex::ComplexFloat;
use futuresdr::prelude::*;

#[test]
fn test_agc_f32_gain_adjustment() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Input signal with power 4.0 (amplitude 2.0)
    let orig: Vec<f32> = vec![2.0; 1000];
    let src = VectorSource::<f32>::new(orig);
    // Target reference power 1.0, initial gain 1.0
    let agc = AgcBuilder::<f32>::new()
        .reference_power(1.0)
        .initial_gain(1.0)
        .adjustment_rate(0.01)
        .build();
    let snk = VectorSink::<f32>::new(1024);

    connect!(fg, src > agc > snk);
    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&snk)?;
    let items = snk.items();
    assert_eq!(items.len(), 1000);

    // After 1000 samples, output amplitude should have converged near 1.0 (power 1.0)
    let last_sample = items.last().unwrap();
    assert!(
        (last_sample - 1.0).abs() < 0.1,
        "Expected ~1.0, got {}",
        last_sample
    );

    Ok(())
}

#[test]
fn test_agc_complex32_power_and_max_gain() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Pure imaginary signal (re=0, im=0.01, power=0.0001)
    let orig: Vec<Complex32> = vec![Complex32::new(0.0, 0.01); 500];
    let src = VectorSource::<Complex32>::new(orig);
    let agc = AgcBuilder::<Complex32>::new()
        .reference_power(1.0)
        .initial_gain(1.0)
        .max_gain(10.0) // Clamp maximum gain to 10.0
        .adjustment_rate(0.1)
        .build();
    let snk = VectorSink::<Complex32>::new(1024);

    connect!(fg, src > agc > snk);
    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&snk)?;
    let items = snk.items();
    assert_eq!(items.len(), 500);

    // Gain was clamped at 10.0, so output magnitude shouldn't exceed 0.1 (0.01 * 10)
    let last_sample = items.last().unwrap();
    assert!(
        last_sample.abs() <= 0.1001,
        "Expected magnitude <= 0.1, got {}",
        last_sample.abs()
    );

    Ok(())
}

#[test]
fn test_agc_squelch_and_zero_inputs() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Input with zeros and weak noise below squelch
    let orig: Vec<f32> = vec![0.0, 0.001, 0.0, 0.002, 0.0];
    let src = VectorSource::<f32>::new(orig);
    let agc = AgcBuilder::<f32>::new()
        .squelch(0.01)
        .reference_power(1.0)
        .build();
    let snk = VectorSink::<f32>::new(1024);

    connect!(fg, src > agc > snk);
    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&snk)?;
    let items = snk.items();
    assert_eq!(items.len(), 5);

    // All samples below squelch should be zeroed
    for &sample in items {
        assert_eq!(sample, 0.0);
    }

    Ok(())
}

#[test]
fn test_agc_gain_lock() -> Result<()> {
    let mut fg = Flowgraph::new();

    let orig: Vec<f32> = vec![2.0; 100];
    let src = VectorSource::<f32>::new(orig);
    // Locked at gain = 3.0
    let agc = AgcBuilder::<f32>::new()
        .initial_gain(3.0)
        .gain_lock(true)
        .reference_power(1.0)
        .build();
    let snk = VectorSink::<f32>::new(1024);

    connect!(fg, src > agc > snk);
    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&snk)?;
    let items = snk.items();
    assert_eq!(items.len(), 100);

    // Every output sample must be exactly 2.0 * 3.0 = 6.0
    for &sample in items {
        assert!((sample - 6.0).abs() < f32::EPSILON);
    }

    Ok(())
}
