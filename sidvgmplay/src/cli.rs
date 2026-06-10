use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sidvgmplay", about = "Render .vgm/.vgz to .wav")]
pub struct Args {
    /// Input .vgm or .vgz file
    pub input: PathBuf,

    /// Output .wav file (default: input name with .wav extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Limit output to this many seconds (default: full duration)
    #[arg(short, long)]
    pub duration: Option<u32>,
}

pub fn parse_args() -> Args {
    Args::parse()
}
