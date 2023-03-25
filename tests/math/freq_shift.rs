use fsdr_blocks::math::FrequencyShifter;
use futuresdr::anyhow::Result;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[test]
fn freq_shift_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let freq_shifter = FrequencyShifter::<f32>::new(2.5, 10.0);

    let orig: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let src = VectorSource::<f32>::new(orig.clone());
    let vect_sink = VectorSinkBuilder::<f32>::new().build();

    connect!(fg,
        src > freq_shifter > vect_sink;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<f32>>(vect_sink).unwrap();
    let snk_0 = snk_0.items();

    assert_eq!(snk_0.len(), orig.len());
    let expected: Vec<f32> = vec![0.0, 0.0, -2.0, 0.0, 4.0, 0.0, -6.0, 0.0, 8.0, 0.0, -10.0];

    // assert!(snk_0
    //     .iter()
    //     .zip(expected.iter())
    //     .all(|(v, e)| (*v - *e).abs() < 0.0000001));
    for (v, e) in snk_0.iter().zip(expected.iter()) {
        assert!(
            (v - e).abs() < 0.0000001,
            "Equality expected. actual: {v}, expected: {e}"
        );
    }

    Ok(())
}

#[test]
fn freq_shift_c32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let freq_shifter = FrequencyShifter::<Complex32>::new(5.0, 10.0);

    let mut orig = Vec::<Complex32>::with_capacity(10);
    let mut expected = Vec::<Complex32>::with_capacity(10);
    for i in 0..10 {
        let f = i as f32;
        let o = Complex32::new(f, 10.0 - f);
        orig.insert(i, o);
        let mut e = o;
        e.re *= (-1i32).pow((i) as u32) as f32;
        e.im *= (-1i32).pow((i) as u32) as f32;
        expected.insert(i, e);
    }
    let orig = orig;
    let expected = expected;

    let src = VectorSource::<Complex32>::new(orig.clone());
    let vect_sink = VectorSinkBuilder::<Complex32>::new().build();

    connect!(fg,
        src > freq_shifter > vect_sink;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<Complex32>>(vect_sink).unwrap();
    let snk_0 = snk_0.items();

    assert_eq!(snk_0.len(), orig.len());
    for (i, (v, e)) in snk_0.iter().zip(expected.iter()).enumerate() {
        assert!(
            (v - e).norm() < 0.000000001,
            "Equality expected. actual[{i}]: {v}, expected[{i}]: {e}"
        );
    }

    Ok(())
}
