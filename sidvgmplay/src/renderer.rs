use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use hound::{WavSpec, WavWriter};
use sidlite_sys::{ChipModel, Sid};
use std::io::{Cursor, Seek, Write};
use std::path::Path;

// 10 ms linear fade-in to mask the DC-offset transient when volume register fires
const FADE_IN_SAMPLES: u32 = 441;

pub fn render(vgm_data: &[u8], output: &Path, duration_limit: Option<u32>) -> Result<()> {
    let mut reader = Cursor::new(vgm_data);

    // VGM header fields
    reader.set_position(0x18);
    let total_samples = reader
        .read_u32::<LittleEndian>()
        .context("read total_samples")?;

    // Data offset at 0x34 is relative to position 0x34
    reader.set_position(0x34);
    let data_rel = reader
        .read_u32::<LittleEndian>()
        .context("read data_offset")?;
    let data_start = 0x34u64 + data_rel as u64;

    reader.set_position(0x78);
    let sid_clock = reader
        .read_u32::<LittleEndian>()
        .context("read sid_clock")?;

    reader.set_position(0x7C);
    let model_byte = reader.read_u8().context("read sid_model")?;
    let make_sid = || {
        let chip_model = if model_byte & 1 == 1 {
            ChipModel::Mos8580
        } else {
            ChipModel::Mos6581
        };
        let mut sid = Sid::new(chip_model);
        sid.set_sampling_parameters(sid_clock, 44100);
        sid
    };

    let total_samples = match duration_limit {
        Some(secs) => total_samples.min(secs * 44100),
        None => total_samples,
    };

    reader.set_position(data_start);

    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav = WavWriter::create(output, spec).context("Failed to create WAV file")?;

    let mut chips = vec![make_sid()];

    let mut written: u32 = 0;
    let mut mix = [0i16; 4096];
    let mut buf = [0i16; 4096];

    loop {
        if written >= total_samples {
            break;
        }
        let cmd = match reader.read_u8() {
            Ok(c) => c,
            Err(_) => break,
        };
        match cmd {
            0xB6 => {
                let chip = reader.read_u8()? as usize;
                let reg = reader.read_u8()?;
                let val = reader.read_u8()?;
                // SID has 32 registers (0x00-0x1F)
                if reg <= 0x1F {
                    while chips.len() <= chip {
                        chips.push(make_sid());
                    }
                    chips[chip].write(reg, val);
                }
            }
            0x61 => {
                let n = reader.read_u16::<LittleEndian>()? as u32;
                clock_chips(
                    &mut chips,
                    n.min(total_samples - written),
                    sid_clock,
                    &mut wav,
                    &mut written,
                    &mut mix,
                    &mut buf,
                )?;
            }
            0x62 => clock_chips(
                &mut chips,
                735u32.min(total_samples - written),
                sid_clock,
                &mut wav,
                &mut written,
                &mut mix,
                &mut buf,
            )?,
            0x63 => clock_chips(
                &mut chips,
                882u32.min(total_samples - written),
                sid_clock,
                &mut wav,
                &mut written,
                &mut mix,
                &mut buf,
            )?,
            0x66 => break,
            0x70..=0x7F => {
                let n = (cmd & 0x0F) as u32 + 1;
                clock_chips(
                    &mut chips,
                    n.min(total_samples - written),
                    sid_clock,
                    &mut wav,
                    &mut written,
                    &mut mix,
                    &mut buf,
                )?;
            }
            _ => {}
        }
    }

    wav.finalize().context("Failed to finalize WAV")?;
    Ok(())
}

fn clock_chips<W: Write + Seek>(
    chips: &mut [Sid],
    samples: u32,
    sid_clock: u32,
    wav: &mut WavWriter<W>,
    total: &mut u32,
    mix: &mut [i16],
    buf: &mut [i16],
) -> Result<()> {
    let mut remaining = samples;
    while remaining > 0 {
        let chunk = remaining.min(mix.len() as u32) as usize;
        let cycles = (chunk as u64 * sid_clock as u64 / 44100) as u32;
        let mut n = chips[0].clock(cycles, mix).min(mix.len());
        for sid in &mut chips[1..] {
            let m = sid.clock(cycles, buf).min(buf.len()).min(n);
            for j in 0..m {
                mix[j] = mix[j].saturating_add(buf[j]);
            }
            n = m;
        }
        for (i, &s) in mix[..n].iter().enumerate() {
            let pos = *total + i as u32;
            let sample = if pos < FADE_IN_SAMPLES {
                (s as i32 * pos as i32 / FADE_IN_SAMPLES as i32) as i16
            } else {
                s
            };
            wav.write_sample(sample)?;
        }
        *total += n as u32;
        remaining -= chunk as u32;
    }
    Ok(())
}
