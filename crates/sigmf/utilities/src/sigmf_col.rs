use anyhow::{Context, Error, Result};
use clap::{arg, Parser, Subcommand};
use sigmf::{DescriptionBuilder, RecordingBuilder};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about="Create and updates collection of SigMF records", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        self.command.execute()
    }
}

#[derive(Subcommand)]
enum Commands {
    #[command(about="Create a collection from given SigMF files", long_about = None)]
    Create {
        #[arg(value_name = "FILE", short, long)]
        output: Option<PathBuf>,
        #[arg(value_name = "AUTHOR", long)]
        author: Option<String>,
        #[arg(value_name = "FILE", required = true)]
        files: Vec<PathBuf>,
    },
    #[command(about="Update a collection", long_about = None)]
    Update {
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },
}

impl Commands {
    fn author(self) -> Option<String> {
        use Commands::*;
        match self {
            Create { author, .. } => author,
            _ => None,
        }
    }

    fn files(&self) -> Result<&Vec<PathBuf>> {
        use Commands::*;
        match self {
            Create { files, .. } => Ok(files),
            _ => unreachable!()
        }
    }

    fn output(&self) -> &PathBuf {
        use Commands::*;
        match self {
            Create { output, .. } => {
                if let Some(output) = output {
                    output
                } else {
                    //PathBuf::from("index.sigmf-meta");
                    unimplemented!()
                }
            }
            _ => unreachable!()
        }
    }

    pub fn execute(self) -> Result<()> {
        use Commands::*;
        match self {
            Create { .. } => self.create_collection(),
            _ => todo!("Not yet implemented"),
        }
    }

    fn create_collection(&self) -> Result<()> {
        let mut collec = DescriptionBuilder::collection();

        for a_file in self.files()? {
            println!("Adding {:?}", a_file);
            let record = RecordingBuilder::from(a_file).compute_sha512()?.build();
            collec.add_stream(record)?;
        }

        let output = self.output();
        // output.set_extension("sigmf-meta");
        collec
            .build()?
            .create(output)
            .with_context(|| format!("Error writing to {}", &output.display()))?;
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = cli.execute() {
        eprintln!("{:#}", err);
    }
}
