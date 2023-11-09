use sigmf::{DatasetFormatBuilder, Description, DescriptionBuilder, SigMFError};

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
    assert_eq!("cu8", global.datatype()?.to_string());
    assert_eq!(
        *global.datatype()?,
        DatasetFormatBuilder::<u8>::complex().build()
    );
    assert_eq!(0, description.annotations()?.len());
    assert_eq!(0, description.captures()?.len());
    Ok(())
}

#[test]
fn parse_example_from_spec() -> Result<(), SigMFError> {
    let metadata = r#"
{
    "global": {
        "core:datatype": "ru8",
        "core:version": "1.0.0",
        "core:dataset": "non-conforming-dataset-01.dat"
    },
    "captures": [
        {
            "core:sample_start": 0,
            "core:header_bytes": 4
        },
        {
            "core:sample_start": 500,
            "core:header_bytes": 4
        }
    ],
    "annotations": []
}"#;
    let description: Description = serde_json::from_str(&metadata)?;
    let global = description.global()?;
    assert_eq!("1.0.0", global.version()?);
    assert_eq!("ru8", global.datatype()?.to_string());
    assert_eq!(
        *global.datatype()?,
        DatasetFormatBuilder::<u8>::real().build()
    );
    assert_eq!(0, description.annotations()?.len());
    assert_eq!(2, description.captures()?.len());
    Ok(())
}

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

#[test]
fn create_simple_description() -> Result<(), SigMFError> {
    let sample_rate = 2_000_000.0;
    let datatype = DatasetFormatBuilder::<u32>::complex()
        .little_endian()
        .build();
    let desc = DescriptionBuilder::from(datatype)
        .sample_rate(sample_rate)
        .build()?;
    let expected = r#"
    {
        "global": {
            "core:datatype": "cu32_le",
            "core:version": "1.0.0",
            "core:sample_rate": 2000000.0
        },
        "captures": [],
        "annotations": []
    }"#;
    let mut expected = expected.to_string();
    expected.retain(|c| !c.is_whitespace());
    let expected_desc: Description = serde_json::from_str(expected.as_str())?;
    assert_eq!(expected_desc, desc);
    let json = serde_json::to_string(&desc)?;
    assert_eq!(expected, json);
    let global = desc.global()?;
    assert_eq!(Some(sample_rate), global.sample_rate);
    assert_eq!(None, global.hw);
    assert_eq!(0, desc.annotations()?.len());
    assert_eq!(0, desc.captures()?.len());
    Ok(())
}

#[test]
fn create_description_with_extensions() -> Result<(), SigMFError> {
    let sample_rate = 2_000_000.0;
    let datatype = DatasetFormatBuilder::<u32>::complex()
        .little_endian()
        .build();
    let desc = DescriptionBuilder::from(datatype)
        .sample_rate(sample_rate)
        .extensions("extension-01", "0.0.5", true)
        .build()?;
    let expected = r#"
    {
        "global": {
            "core:datatype": "cu32_le",
            "core:version": "1.0.0",
            "core:sample_rate": 2000000.0,
            "core:extensions" : [
                {
                    "name": "extension-01",
                    "version": "0.0.5",
                    "optional": true
                }
            ]
        },
        "captures": [],
        "annotations": []
    }"#;
    let mut expected = expected.to_string();
    expected.retain(|c| !c.is_whitespace());
    let expected_desc: Description = serde_json::from_str(expected.as_str())?;
    assert_eq!(expected_desc, desc);
    let json = serde_json::to_string(&desc)?;
    assert_eq!(expected, json);
    let global = desc.global()?;
    assert_eq!(Some(sample_rate), global.sample_rate);
    assert_eq!(None, global.hw);
    assert_eq!(0, desc.annotations()?.len());
    assert_eq!(0, desc.captures()?.len());
    Ok(())
}

#[test]
fn parse_antenna() -> Result<(), SigMFError> {
    let metadata = r#"{
    "global": {
        "core:datatype": "cu8",
        "core:version": "1.0.0",
        "core:extensions" : [
            {
                "name": "antenna",
                "version": "1.0.0",
                "optional": false
            }
        ],
        "antenna:model": "ARA CSB-16"
    },
    "captures": [],
    "annotations": []
}
"#;
    let description: Description = serde_json::from_str(&metadata)?;
    let global = description.global()?;
    assert_eq!("1.0.0", global.version()?);
    assert_eq!("cu8", global.datatype()?.to_string());
    assert_eq!(0, description.annotations()?.len());
    assert_eq!(0, description.captures()?.len());
    let antenna_desc = &global.antenna;
    assert_eq!("ARA CSB-16", antenna_desc.model()?);
    Ok(())
}
