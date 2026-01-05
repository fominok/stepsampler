use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{Result, bail};
use argh::FromArgs;
use stepsampler::Config;

#[derive(FromArgs, Debug)]
/// Converts multiple samples to evenly sized grid.
struct Args {
    #[argh(option, short = 'o', description = "output wav file")]
    output: PathBuf,

    #[argh(
        option,
        short = 'r',
        description = "output sampling rate",
        default = "stepsampler::DEFAULT_OUT_RATE"
    )]
    out_rate: u32,

    #[argh(
        option,
        short = 't',
        description = "silence threshold",
        default = "stepsampler::DEFAULT_SILENCE_THRESHOLD"
    )]
    silence_threshold: f32,

    #[argh(switch, short = 's', description = "produce stereo output")]
    stereo: bool,

    #[argh(positional, description = "input wav files")]
    input: Vec<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();

    let args: Args = argh::from_env();

    if args.input.is_empty() {
        bail!("no input files provided");
    }

    let mut files = Vec::new();

    for file in args.input {
        files.push(File::open(file)?);
    }

    let config = Config {
        silence_threshold: args.silence_threshold,
        stereo: args.stereo,
        rate: args.out_rate,
    };

    println!(
        "Output sample will contain {} segments; {config}",
        files.len()
    );

    let output_bytes = stepsampler::step_sample(config, files)?;

    fs::write(args.output, output_bytes)?;

    Ok(())
}
