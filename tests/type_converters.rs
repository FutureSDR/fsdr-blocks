use fsdr_blocks::type_converters::*;
use futuresdr::blocks::{VectorSink, VectorSource};
use futuresdr::prelude::*;

#[test]
fn convert_u8_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let convert_u8_f32 = TypeConvertersBuilder::convert::<u8, f32>().build();

    let orig: Vec<u8> = vec![1, 0, 255, 42, 53, 89, 75];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSink::<f32>::new(1024);

    connect!(fg,
        src > convert_u8_f32 > vect_sink;
    );
    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&vect_sink)?;
    let v = snk.items();

    assert_eq!(v.len(), orig.len());
    for (v_before, v_after) in orig.iter().zip(v) {
        assert!(((*v_after) - (*v_before as f32)).abs() < f32::EPSILON);
    }

    Ok(())
}

#[test]
fn test_signed_zero_centered_scaled_converters() -> Result<()> {
    // Test i8 <-> f32
    assert_eq!(ScaledConverterBuilder::<i8, f32>::convert(&0), 0.0);
    assert_eq!(ScaledConverterBuilder::<i8, f32>::convert(&127), 1.0);
    assert_eq!(ScaledConverterBuilder::<i8, f32>::convert(&-127), -1.0);
    assert_eq!(ScaledConverterBuilder::<f32, i8>::convert(&0.0), 0);
    assert_eq!(ScaledConverterBuilder::<f32, i8>::convert(&1.0), 127);
    assert_eq!(ScaledConverterBuilder::<f32, i8>::convert(&-1.0), -127);
    // Boundary clamp check
    assert_eq!(ScaledConverterBuilder::<f32, i8>::convert(&10.0), 127);
    assert_eq!(ScaledConverterBuilder::<f32, i8>::convert(&-10.0), -128);

    // Test i16 <-> f32
    assert_eq!(ScaledConverterBuilder::<i16, f32>::convert(&0), 0.0);
    assert_eq!(ScaledConverterBuilder::<i16, f32>::convert(&32767), 1.0);
    assert_eq!(ScaledConverterBuilder::<i16, f32>::convert(&-32767), -1.0);
    assert_eq!(ScaledConverterBuilder::<f32, i16>::convert(&0.0), 0);
    assert_eq!(ScaledConverterBuilder::<f32, i16>::convert(&1.0), 32767);
    assert_eq!(ScaledConverterBuilder::<f32, i16>::convert(&-1.0), -32767);
    assert_eq!(ScaledConverterBuilder::<f32, i16>::convert(&10.0), 32767);

    // Test i32 <-> f32
    assert_eq!(ScaledConverterBuilder::<i32, f32>::convert(&0), 0.0);
    assert_eq!(ScaledConverterBuilder::<i32, f32>::convert(&i32::MAX), 1.0);
    assert_eq!(ScaledConverterBuilder::<f32, i32>::convert(&0.0), 0);

    Ok(())
}

#[test]
fn test_unsigned_midpoint_scaled_converters() -> Result<()> {
    // Test u8 <-> f32 (128 is 0.0)
    assert_eq!(ScaledConverterBuilder::<u8, f32>::convert(&128), 0.0);
    assert_eq!(ScaledConverterBuilder::<u8, f32>::convert(&0), -1.0);
    assert_eq!(ScaledConverterBuilder::<f32, u8>::convert(&0.0), 128);
    assert_eq!(ScaledConverterBuilder::<f32, u8>::convert(&-1.0), 0);
    assert_eq!(ScaledConverterBuilder::<f32, u8>::convert(&1.0), 255);

    // Test u16 <-> f32 (32768 is 0.0)
    assert_eq!(ScaledConverterBuilder::<u16, f32>::convert(&32768), 0.0);
    assert_eq!(ScaledConverterBuilder::<u16, f32>::convert(&0), -1.0);
    assert_eq!(ScaledConverterBuilder::<f32, u16>::convert(&0.0), 32768);

    Ok(())
}

#[test]
fn test_scaled_conversion_in_flowgraph() -> Result<()> {
    let mut fg = Flowgraph::new();

    let orig: Vec<i16> = vec![0, 32767, -32767, 16383, -16383];
    let src = VectorSource::<i16>::new(orig);
    let conv = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
    let snk = VectorSink::<f32>::new(1024);

    connect!(fg, src > conv > snk);
    let fg = Runtime::new().run(fg)?;

    let snk = fg.block(&snk)?;
    let items = snk.items();
    assert_eq!(items.len(), 5);
    assert_eq!(items[0], 0.0);
    assert!((items[1] - 1.0).abs() < 1e-4);
    assert!((items[2] - (-1.0)).abs() < 1e-4);
    assert!((items[3] - 0.5).abs() < 1e-3);
    assert!((items[4] - (-0.5)).abs() < 1e-3);

    Ok(())
}
