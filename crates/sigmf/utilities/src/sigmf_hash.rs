use anyhow::{Context, Result};
use clap::{arg, Parser, Subcommand};
use sigmf::RecordingBuilder;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about="Check and update hashes on sigmf files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about="Verify the hash of a dataset", long_about = None)]
    Check {
        #[arg(value_name = "FILE", required = true)]
        files: Vec<PathBuf>,
    },
    #[command(about="Recompute and update the hash of a dataset", long_about = None)]
    Update {
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    use Commands::*;
    match cli {
        Cli {
            command: Check { files },
        } => check(files),
        Cli {
            command: Update { files },
        } => update(files),
    }
}

fn check(files: Vec<PathBuf>) {
    for a_file in files {
        if let Err(err) = check_sigmf(a_file) {
            eprintln!("{:#}", err);
        }
    }
}

fn check_sigmf(basename: PathBuf) -> Result<()> {
    let mut record = RecordingBuilder::from(&basename)
        .compute_sha512()
        .with_context(|| format!("Computing sha512 of {}", basename.display()))?
        .build();
    let computed_sha512 = record.hash()?.clone();
    let desc = record.load_description()?;
    let expected_sha512 = desc.global()?.sha512.as_ref().expect("sha512 not present");
    if expected_sha512.eq(&computed_sha512) {
        println!("Hash match");
    } else {
        println!("{}", expected_sha512);
        println!("{}", computed_sha512);
        println!("Hash doesn't match");
    }
    Ok(())
}

fn update(files: Vec<PathBuf>) {
    for a_file in files {
        if let Err(err) = update_sigmf(a_file) {
            eprintln!("{:#}", err);
        }
    }
}

fn update_sigmf(basename: PathBuf) -> Result<()> {
    let mut record = RecordingBuilder::from(&basename)
        .compute_sha512()
        .with_context(|| format!("While computing sha512 of {}", basename.display()))?
        .build();
    let computed_sha512 = record.hash()?.clone();
    let mut desc = record.load_description()?;
    let expected_sha512 = desc.global()?.sha512.as_ref();
    let mut need_update = true;
    if let Some(expected_sha512) = expected_sha512 {
        need_update = !expected_sha512.eq(&computed_sha512);
    }
    if need_update {
        let mut basename = basename.to_path_buf();
        basename.set_extension("sigmf-meta");
        desc.global_mut()?.sha512 = Some(computed_sha512.to_string());
        desc.create_pretty(&basename)
            .with_context(|| format!("Error writing to {}", &basename.display()))?;
    }
    Ok(())
}
