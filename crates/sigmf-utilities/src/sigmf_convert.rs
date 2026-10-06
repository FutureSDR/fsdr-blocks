use anyhow::anyhow;
use clap::Parser;
use fsdr_blocks::sigmf::DatasetFormat;
use fsdr_blocks::sigmf::DatasetFormat::*;
use fsdr_blocks::sigmf::{SigMFSinkBuilder, SigMFSourceBuilder};
use fsdr_blocks::type_converters::TypeConvertersBuilder;
use futuresdr::blocks::Apply;
use futuresdr::blocks::TagDebug;
use futuresdr::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about="Lossly Convert the type of data by going through float32", long_about = None)]
struct Cli {
    #[arg(value_name = "INPUT", required = true)]
    input: PathBuf,
    #[arg(value_name = "DATATYPE", required = true)]
    target: DatasetFormat,
    #[arg(value_name = "OUTPUT", required = true)]
    output: PathBuf,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        let mut fg = Flowgraph::new();

        let mut src_builder = SigMFSourceBuilder::from(&self.input);
        let src = fg.add_async(src_builder.build::<f32>().await?).await?;

        let snk = SigMFSinkBuilder::from(self.output);

        match self.target {
            RI8 => {
                let conv = TypeConvertersBuilder::lossy_scale_convert_f32_i8().build();
                let snk = snk.datatype(self.target).build::<i8>().await?;
                connect!(fg, src > conv > snk);
            }
            RU8 => {
                let conv = TypeConvertersBuilder::lossy_scale_convert_f32_u8().build();
                let snk = snk.datatype(self.target).build::<u8>().await?;
                connect!(fg, src > conv > snk);
            }
            Rf32Be | Rf32Le => {
                let conv: Apply<fn(&f32) -> f32, f32, f32> = Apply::new(|x: &f32| *x);
                let snk = snk.datatype(self.target).build::<f32>().await?;
                connect!(fg, src > conv > snk);
            }
            Rf64Be | Rf64Le => {
                let conv = TypeConvertersBuilder::convert::<f32, f64>().build();
                let snk = snk.datatype(self.target).build::<f64>().await?;
                connect!(fg, src > conv > snk);
            }
            Ri16Be | Ri16Le => {
                let conv = TypeConvertersBuilder::lossy_scale_convert_f32_i16().build();
                let snk = snk.datatype(self.target).build::<i16>().await?;
                connect!(fg, src > conv > snk);
            }
            _ => return Err(anyhow!("Unsupported target type: {}", self.target)),
        };

        let tag_dbg = TagDebug::<f32>::new("debugger");
        connect!(fg, src > tag_dbg);

        Runtime::new().run(fg)?;
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = block_on(cli.execute()) {
        eprintln!("{:#}", err);
    }
}
