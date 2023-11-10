use anyhow::{Context, Result};
use clap::{arg, Parser, Subcommand};
use sha2::{Digest, Sha512};
use sigmf::Description;
use std::fs::File;
use std::io::{BufReader,Read};
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
    for mut a_file in files {
        if let Err(err) = check_sigmf(&mut a_file) {
            eprintln!("{:#}", err);
        }
    }
}

fn get_desc(basename: &mut PathBuf) -> Result<Description> {
    basename.set_extension("sigmf-meta");
    let path = basename.as_path();
    let meta_file =
        File::open(path).with_context(|| format!("Error opening {}", path.display()))?;
    let rdr = BufReader::new(meta_file);
    let desc: Result<Description, serde_json::Error> = serde_json::from_reader(rdr);
    Ok(desc?)
}

fn compute_sha512(basename: &mut PathBuf) -> Result<impl AsRef<[u8]>> {
    basename.set_extension("sigmf-data");
    let path = basename.as_path();
    let mut data_file =
        File::open(path).with_context(|| format!("Error opening {}", path.display()))?;
    let mut hasher = Sha512::new();
    let mut buffer = [0; 1024];

    loop {
        let count = data_file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize())
}

fn check_sigmf(basename: &mut PathBuf) -> Result<()> {
    let desc = get_desc(basename)?;
    let expected_sha512 = desc.global()?.sha512.as_ref().expect("sha512 not present");
    let computed_sha512 = compute_sha512(basename)?;
    let computed_sha512 = hex::encode(computed_sha512);
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
    for mut a_file in files {
        if let Err(err) = update_sigmf(&mut a_file) {
            eprintln!("{:#}", err);
        }
    }
}

fn update_sigmf(basename: &mut PathBuf) -> Result<()> {
    let mut desc = get_desc(basename)?;
    let computed_sha512 = compute_sha512(basename)?;
    let computed_sha512 = hex::encode(computed_sha512);
    desc.global_mut()?.sha512 = Some(computed_sha512);

    basename.set_extension("sigmf-meta");
    let mut data_file =
        File::create(&basename).with_context(|| format!("Error opening {}", &basename.display()))?;
    serde_json::to_writer(data_file, &desc)?;
    Ok(())
}