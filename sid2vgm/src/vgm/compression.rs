use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn compress_vgm(vgm_data: &[u8], output_path: &Path) -> io::Result<()> {
    let output = File::create(output_path)?;
    let mut encoder = GzEncoder::new(output, Compression::best());
    encoder.write_all(vgm_data)?;
    encoder.finish()?;
    Ok(())
}
