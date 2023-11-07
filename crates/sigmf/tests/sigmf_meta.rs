use sigmf::{Description, SigMFError};

#[test]
fn parse_mandatory() -> Result<(), SigMFError> {
    let metadata = r#"{
    "global": {
        "core:datatype": "cu8",
        "core:version": "1.0.0"
    },
    "captures": [],
    "annotations": []
}
"#;
    let description: Description = serde_json::from_str(&metadata)?;
    let global = description.global()?;
    assert_eq!("1.0.0", global.version()?);
    assert_eq!("cu8", global.datatype()?);
    // assert_eq!(0, description.annotations())
    Ok(())
}

// #[test]
// fn parse_example_from_spec() -> Result<()> {
//     let metadata = r#"
// {
//     "global": {
//         "core:datatype": "cu8",
//         "core:version": "1.0.0",
//         "core:dataset": "non-conforming-dataset-01.dat"
//     },
//     "captures": [
//         {
//             "core:sample_start": 0,
//             "core:header_bytes": 4,
//         },
//         {
//             "core:sample_start": 500,
//             "core:header_bytes": 4,
//         }
//     ],
//     "annotations": []
// }"#;
//     Ok(())
// }



// {
//     "global": {
//         "core:datatype": "cf32_le",
//         "core:sample_rate": 2000000,
//         "core:hw": "HachRF(tm) One with bi-bands double J antenna",
//         "core:author": "Loïc Fejoz",
//         "core:version": "1.0.0",
//         "core:description": "GQRX recording of VHF APRS"
//     },
//     "captures": [
//         {
//             "core:sample_start": 0,
//             "core:frequency": 145171400,
//             "core:datetime": "2023-11-04T10:17:25Z"
//         }
//     ],
//     "annotations": []
// }