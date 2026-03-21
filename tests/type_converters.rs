use fsdr_blocks::type_converters::*;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Result;
use futuresdr::runtime::Runtime;

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
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    assert_eq!(v.len(), orig.len());
    for (v_before, v_after) in orig.iter().zip(v) {
        assert!(((*v_after) - (*v_before as f32)).abs() < f32::EPSILON);
    }

    Ok(())
}

#[test]
fn scale_convert_u8_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let convert_u8_f32 = TypeConvertersBuilder::scale_convert::<u8, f32>().build();

    let orig: Vec<u8> = vec![0, 127, 128, 255];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink = VectorSink::<f32>::new(1024);

    connect!(fg,
        src > convert_u8_f32 > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    assert_eq!(v.len(), orig.len());
    // 0 -> -1.0
    // 127.5 -> 0.0 (but u8 is integer, so 127 or 128)
    // 255 -> 1.0

    assert!((v[0] - (-1.0)).abs() < 1e-6);
    assert!((v[3] - 1.0).abs() < 1e-6);

    Ok(())
}

// #[test]
// fn convert_u8_f32_with_scale_3() -> Result<()> {
//     const SCALE_FACTOR: f32 = 3.0;
//     let mut fg = Flowgraph::new();

//     let convert_u8_f32 = TypeConvertersBuilder::convert::<u8, f32>()
//         .scale(SCALE_FACTOR)
//         .build();

//     let orig: Vec<u8> = vec![1, 0, 255, 42, 53, 89, 75];
//     let src = VectorSource::<u8>::new(orig.clone());
//     let vect_sink = VectorSink::<f32>::new(1024);

//     connect!(fg,
//         src > convert_u8_f32 > vect_sink;
//     );
//     Runtime::new().run(fg)?;

//     let snk = vect_sink.get()?;
//     let v = snk.items();

//     assert_eq!(v.len(), orig.len());
//     for (v_before, v_after) in orig.iter().zip(v) {
//         assert!(((*v_after as f32) - SCALE_FACTOR*(*v_before as f32)).abs() < f32::EPSILON);
//     }

//     Ok(())
// }
