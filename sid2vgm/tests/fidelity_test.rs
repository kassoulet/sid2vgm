#[path = "songlengths.rs"]
mod songlengths;

use byteorder::{LittleEndian, ReadBytesExt};
use hound::{WavSpec, WavWriter};
use sidlite_sys::{ChipModel, Sid};
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::process::Command;

#[test]
fn test_audio_fidelity() {
    let sid_path = "tests/sids/Andropolis.sid";
    let vgm_path = "tests/fidelity.vgm";
    let sid_wav_path = "tests/sid_ref.wav";
    let vgm_wav_path = "tests/vgm_render.wav";

    let sl = songlengths::SongLengths::load("tests/sids/Songlengths.txt");
    let duration_secs = sl
        .duration_secs(std::path::Path::new(sid_path))
        .expect("Songlengths.txt entry required for test — no fallback to 60s");

    fs::create_dir_all("tests").ok();

    // 1. Render reference SID to WAV using sidplayfp
    let status = Command::new("sidplayfp")
        .arg(format!("-t{}", duration_secs))
        .arg(format!("-w{}", sid_wav_path))
        .arg(sid_path)
        .status()
        .expect("Failed to execute sidplayfp");
    assert!(status.success(), "sidplayfp failed to render reference");

    // 2. Convert SID to VGM using our tool
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg(sid_path)
        .arg("-o")
        .arg(vgm_path)
        .arg("-d")
        .arg(duration_secs.to_string())
        .status()
        .expect("Failed to execute sid2vgm");
    assert!(status.success(), "sid2vgm failed");

    // 3. Render VGM to WAV using sidlite-sys
    render_vgm_to_wav(vgm_path, vgm_wav_path, duration_secs);

    // 4. Compare WAVs
    compare_wavs(sid_wav_path, vgm_wav_path);

    // Cleanup
    fs::remove_file(vgm_path).ok();
    fs::remove_file(sid_wav_path).ok();
    fs::remove_file(vgm_wav_path).ok();
}

fn render_vgm_to_wav(vgm_path: &str, wav_path: &str, duration_secs: u32) {
    let mut vgm_data = Vec::new();
    File::open(vgm_path)
        .unwrap()
        .read_to_end(&mut vgm_data)
        .unwrap();
    let mut reader = Cursor::new(vgm_data);

    // SID clock at 0x78, data starts at 0x80
    reader.set_position(0x78);
    let sid_clock = reader.read_u32::<LittleEndian>().unwrap();
    reader.set_position(0x80);

    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create(wav_path, spec).unwrap();

    let mut sid = Sid::new(ChipModel::Mos8580);
    sid.set_sampling_parameters(sid_clock, 44100);

    let mut total_samples = 0;
    let target_samples = duration_secs * 44100;
    let mut sample_buf = [0i16; 4096];

    while total_samples < target_samples {
        let cmd = match reader.read_u8() {
            Ok(c) => c,
            Err(_) => break,
        };

        match cmd {
            0xB6 => {
                let _chip = reader.read_u8().unwrap();
                let reg = reader.read_u8().unwrap();
                let val = reader.read_u8().unwrap();
                sid.write(reg, val);
            }
            0x61 => {
                let samples = reader.read_u16::<LittleEndian>().unwrap() as u32;
                clock_sid(
                    &mut sid,
                    samples,
                    sid_clock,
                    &mut writer,
                    &mut total_samples,
                    &mut sample_buf,
                );
            }
            0x62 => clock_sid(
                &mut sid,
                735,
                sid_clock,
                &mut writer,
                &mut total_samples,
                &mut sample_buf,
            ),
            0x63 => clock_sid(
                &mut sid,
                882,
                sid_clock,
                &mut writer,
                &mut total_samples,
                &mut sample_buf,
            ),
            0x66 => break,
            0x70..=0x7F => {
                let samples = (cmd & 0x0F) as u32 + 1;
                clock_sid(
                    &mut sid,
                    samples,
                    sid_clock,
                    &mut writer,
                    &mut total_samples,
                    &mut sample_buf,
                );
            }
            _ => {}
        }
    }

    if total_samples < target_samples {
        clock_sid(
            &mut sid,
            target_samples - total_samples,
            sid_clock,
            &mut writer,
            &mut total_samples,
            &mut sample_buf,
        );
    }

    writer.finalize().unwrap();
}

fn clock_sid<W: std::io::Write + std::io::Seek>(
    sid: &mut Sid,
    samples: u32,
    sid_clock: u32,
    writer: &mut WavWriter<W>,
    total: &mut u32,
    buf: &mut [i16],
) {
    let mut remaining = samples;
    while remaining > 0 {
        let to_clock = remaining.min(buf.len() as u32) as usize;
        let cycles = (to_clock as u64 * sid_clock as u64 / 44100) as u32;
        let n = sid.clock(cycles, buf).min(buf.len());
        for sample in &buf[..n] {
            writer.write_sample(*sample).unwrap();
        }
        *total += n as u32;
        remaining -= to_clock as u32;
    }
}

fn compare_wavs(ref_path: &str, test_path: &str) {
    let mut ref_wav = hound::WavReader::open(ref_path).unwrap();
    let mut test_wav = hound::WavReader::open(test_path).unwrap();

    let ref_samples: Vec<i16> = ref_wav.samples::<i16>().map(|s| s.unwrap()).collect();
    let test_samples: Vec<i16> = test_wav.samples::<i16>().map(|s| s.unwrap()).collect();

    let len = ref_samples.len().min(test_samples.len());
    let mut sum_sq_diff = 0.0;

    for i in 0..len {
        let diff = (ref_samples[i] as f64 - test_samples[i] as f64) / 32768.0;
        sum_sq_diff += diff * diff;
    }

    let rms_error = (sum_sq_diff / len as f64).sqrt();
    println!("RMS Error: {}", rms_error);

    assert!(rms_error < 0.5, "RMS error too high: {}", rms_error);
}
