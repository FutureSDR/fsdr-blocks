use fsdr_blocks::type_converters::*;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
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
    let vect_sink = VectorSinkBuilder::<f32>::new().build();

    connect!(fg,
        src > convert_u8_f32 > vect_sink;
    );
    fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<f32>>(vect_sink).unwrap();
    let v = snk.items();

    assert_eq!(v.len(), orig.len());
    for (v_before, v_after) in orig.iter().zip(v) {
        assert!(((*v_after) - (*v_before as f32)).abs() < f32::EPSILON);
    }

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
//     let vect_sink = VectorSinkBuilder::<f32>::new().build();

//     connect!(fg,
//         src > convert_u8_f32 > vect_sink;
//     );
//     fg = Runtime::new().run(fg)?;

//     let snk = fg.kernel::<VectorSink<f32>>(vect_sink).unwrap();
//     let v = snk.items();

//     assert_eq!(v.len(), orig.len());
//     for (v_before, v_after) in orig.iter().zip(v) {
//         assert!(((*v_after as f32) - SCALE_FACTOR*(*v_before as f32)).abs() < f32::EPSILON);
//     }

//     Ok(())
// }
