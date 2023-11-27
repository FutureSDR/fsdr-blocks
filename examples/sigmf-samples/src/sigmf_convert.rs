use clap::{arg, Parser};
use fsdr_blocks::sigmf::DatasetFormat;
use fsdr_blocks::sigmf::DatasetFormat::*;
use fsdr_blocks::{
    sigmf::{SigMFSinkBuilder, SigMFSourceBuilder},
    type_converters::TypeConvertersBuilder,
};
use futuresdr::{
    anyhow::{anyhow, Context, Result},
    blocks::Apply,
    num_complex::{Complex, Complex32},
    runtime::{Flowgraph, Runtime},
};
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

        let mut src = SigMFSourceBuilder::from(&self.input);
        let src = src.build::<f32>().await?;
        let src = fg.add_block(src);

        let snk = SigMFSinkBuilder::from(self.output);

        let (conv, snk) = match self.target {
            RI8 => (
                fg.add_block(TypeConvertersBuilder::lossy_scale_convert_f32_i8().build()),
                fg.add_block(snk.datatype(self.target).build::<i8>().await?),
            ),
            RU8 => (
                fg.add_block(TypeConvertersBuilder::lossy_scale_convert_f32_u8().build()),
                fg.add_block(snk.datatype(self.target).build::<u8>().await?),
            ),
            Rf32Be | Rf32Le => (
                fg.add_block(Apply::new(|x: &f32| *x)),
                fg.add_block(snk.datatype(self.target).build::<f32>().await?),
            ),
            Rf64Be | Rf64Le => (
                fg.add_block(TypeConvertersBuilder::convert::<f32, f64>().build()),
                fg.add_block(snk.datatype(self.target).build::<f64>().await?),
            ),
            Ri16Be | Ri16Le => (
                fg.add_block(TypeConvertersBuilder::lossy_scale_convert_f32_i16().build()),
                fg.add_block(snk.datatype(self.target).build::<i16>().await?),
            ),
            // Ri32Be | Ri32Le  => {
            //     fg.add_block(TypeConvertersBuilder::convert::<f32, i32>().build())
            // }
            // Ru16Be | Ru16Le  => {
            //     fg.add_block(TypeConvertersBuilder::convert::<f32, u16>().build())
            // }
            // Ru32Be | Ru32Le  => {
            //     fg.add_block(TypeConvertersBuilder::convert::<f32, u32>().build())
            // }
            _ => return Err(anyhow!("Unsupported target type: {}", self.target)),
        };
        fg.connect_stream(src, "out", conv, "in")
            .with_context(|| "src->conv")?;
        fg.connect_stream(conv, "out", snk, "in")
            .with_context(|| "conv->snk")?;

        Runtime::new().run(fg)?;
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = futuresdr::futures::executor::block_on(cli.execute()) {
        eprintln!("{:#}", err);
    }
}
