mod cli;
mod convert;
mod sid;
mod songlengths;
mod trace;
mod vgm;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let args = cli::parse_args();

    let input = args.input;
    let output = args.output.unwrap_or_else(|| {
        let mut out = input.clone();
        out.set_extension("vgm");
        out
    });

    let duration_secs = match args.duration {
        Some(d) => d,
        None => {
            let input_dir = input.parent().unwrap_or(std::path::Path::new("."));
            let sl_path = songlengths::find_songlengths(input_dir)
                .with_context(|| format!(
                    "No Songlengths.txt found searching from {}. Use --duration to set a duration.",
                    input_dir.display()
                ))?;
            let sl = songlengths::SongLengths::load(&sl_path);
            sl.duration_secs(&input, args.subtune).with_context(|| {
                format!(
                    "'{}' not found in {}. Use --duration to set a duration.",
                    input.file_name().unwrap_or_default().to_string_lossy(),
                    sl_path.display()
                )
            })?
        }
    };

    let mut converter = convert::Converter::new()?;

    println!("Input:     {}", input.display());
    println!("Output:    {}", output.display());
    println!(
        "Subtune:   {}",
        if args.subtune == 0 {
            "Default".into()
        } else {
            args.subtune.to_string()
        }
    );
    println!("Duration:  {}s", duration_secs);

    let is_vgz = output.extension().is_some_and(|ext| ext == "vgz");
    let target_path = if is_vgz {
        let mut temp = output.clone();
        temp.set_extension("vgm.tmp");
        temp
    } else {
        output.clone()
    };

    let pal_override = match (args.pal, args.ntsc) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    };

    let stats = converter.convert(
        &input,
        &target_path,
        args.subtune,
        duration_secs,
        args.loop_point,
        pal_override,
    )?;

    if is_vgz {
        println!("Compressing to .vgz...");
        let vgm_data = std::fs::read(&target_path)?;
        vgm::compression::compress_vgm(&vgm_data, &output)?;
        std::fs::remove_file(&target_path)?;
    }

    if args.stats {
        println!("--- Statistics ---");
        println!("Events:    {}", stats.event_count);
        let duration_secs = stats.total_samples as f64 / 44100.0;
        let mins = (duration_secs / 60.0).floor() as u32;
        let secs = (duration_secs % 60.0).round() as u32;
        println!("Length:    {}m{:02}s", mins, secs);
        let final_size = std::fs::metadata(&output)?.len();
        println!("Final size: {} KB", final_size / 1024);
    }

    println!("Conversion complete.");

    Ok(())
}
