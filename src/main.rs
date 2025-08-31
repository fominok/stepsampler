use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{Result, bail};
use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Converts multiple samples to evenly sized grid.
struct Args {
    #[argh(option, short = 'o', description = "output wav file")]
    output: PathBuf,

    #[argh(positional, description = "input wav files")]
    input: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    if args.input.is_empty() {
        bail!("no input files provided");
    }

    let mut files = Vec::new();

    for file in args.input {
        files.push(File::open(file)?);
    }

    println!("Output sample will contain {} segments", files.len());

    let output_bytes = stepsampler::step_sample(files)?;

    fs::write(args.output, output_bytes)?;

    Ok(())
}
