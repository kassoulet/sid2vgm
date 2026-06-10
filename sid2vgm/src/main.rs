mod cli;
mod convert;
mod sid;
mod trace;
mod vgm;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse_args();

    let input = args.input;
    let output = args.output.unwrap_or_else(|| {
        let mut out = input.clone();
        out.set_extension("vgm");
        out
    });

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
    println!("Duration:  {}s", args.duration);

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
        args.duration,
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
