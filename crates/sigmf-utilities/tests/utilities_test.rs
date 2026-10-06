use sigmf::{DatasetFormat, DescriptionBuilder, RecordingBuilder};
use std::fs::File;
use std::io::Write;

#[test]
fn test_sigmf_utilities_hash_and_description() -> anyhow::Result<()> {
    let dir = std::env::temp_dir().join("sigmf_util_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;

    let base = dir.join("test_record");
    let data_file = base.with_extension("sigmf-data");
    let meta_file = base.with_extension("sigmf-meta");

    // Write some data
    let mut f = File::create(&data_file)?;
    f.write_all(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    drop(f);

    // Write description
    let desc = DescriptionBuilder::from(DatasetFormat::RU8).build()?;
    desc.create_pretty(&meta_file)?;

    // Compute hash
    let record = RecordingBuilder::from(&base).compute_sha512()?.build();
    let hash = record.hash()?.clone();
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 128); // SHA-512 hex string length

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
