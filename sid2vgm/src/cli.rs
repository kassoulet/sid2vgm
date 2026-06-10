use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sid2vgm", author, version, about = "Convert .sid to .vgm", long_about = None)]
pub struct Args {
    /// Input .sid file
    pub input: PathBuf,

    /// Output .vgm or .vgz file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Subtune to convert (0 = default)
    #[arg(short, long, default_value_t = 0)]
    pub subtune: u16,

    /// Duration in seconds to capture (default: from Songlengths.txt lookup)
    #[arg(short, long)]
    pub duration: Option<u32>,

    /// Force PAL timing
    #[arg(long)]
    pub pal: bool,

    /// Force NTSC timing
    #[arg(long)]
    pub ntsc: bool,

    /// Loop point in seconds
    #[arg(long)]
    pub loop_point: Option<f64>,

    /// Dump statistics
    #[arg(long)]
    pub stats: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
