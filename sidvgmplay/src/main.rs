mod cli;
mod renderer;
#[cfg(test)]
mod security_test;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;

fn main() -> Result<()> {
    let args = cli::parse_args();

    let output = args.output.unwrap_or_else(|| {
        let mut out = args.input.clone();
        out.set_extension("wav");
        out
    });

    println!("Input:  {}", args.input.display());
    println!("Output: {}", output.display());

    let raw = std::fs::read(&args.input)
        .with_context(|| format!("Failed to read {}", args.input.display()))?;

    // Decompress .vgz (gzip) transparently
    let vgm_data = if args.input.extension().is_some_and(|e| e == "vgz") || is_gzip(&raw) {
        let mut decoder = GzDecoder::new(raw.as_slice());
        let mut buf = Vec::new();
        decoder
            .read_to_end(&mut buf)
            .context("Failed to decompress VGZ")?;
        buf
    } else {
        raw
    };

    renderer::render(&vgm_data, &output, args.duration)?;

    println!("Done.");
    Ok(())
}

fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}
