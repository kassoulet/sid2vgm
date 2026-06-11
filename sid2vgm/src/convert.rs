use crate::sid::loader::read_sid_model;
use crate::sid::player::Player;
use crate::vgm::commands::VgmCommand;
use crate::vgm::header::{Clock, VgmHeader};
use crate::vgm::writer::VgmWriter;
use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

const CHUNK_CYCLES: u64 = 1_000_000;

pub struct Converter {
    player: Player,
}

pub struct ConversionStats {
    pub event_count: usize,
    pub total_samples: u32,
}

impl Converter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            player: Player::new()?,
        })
    }

    pub fn convert(
        &mut self,
        input: &Path,
        output: &Path,
        subtune: u16,
        duration_secs: u32,
        loop_point: Option<f64>,
        pal_override: Option<bool>,
    ) -> Result<ConversionStats> {
        self.player
            .load_file(input, subtune)
            .context("Failed to load SID file")?;

        let is_pal = pal_override.unwrap_or_else(|| self.player.is_pal());
        let sid_clock = if is_pal { Clock::Pal } else { Clock::Ntsc };
        let clock_rate = sid_clock as u64;

        let sid_model = read_sid_model(input);

        let target_sample_rate: u64 = 44100;
        let total_cycles = duration_secs as u64 * clock_rate;

        let mut all_events = Vec::new();
        let mut current_cycle = 0;

        while current_cycle < total_cycles {
            let to_play = (total_cycles - current_cycle).min(CHUNK_CYCLES) as u32;
            let events = self.player.step(to_play).context("Emulation error")?;
            all_events.extend(events);
            current_cycle += to_play as u64;
        }

        let event_count = all_events.len();

        let mut vgm_events: Vec<(u32, VgmCommand)> = all_events
            .into_iter()
            .map(|e| {
                let sample = (e.cycle * target_sample_rate / clock_rate) as u32;
                (
                    sample,
                    VgmCommand::SidWrite {
                        chip: e.chip,
                        reg: e.register,
                        val: e.value,
                    },
                )
            })
            .collect();

        vgm_events.sort_by_key(|(s, _)| *s);

        let vgm_file = File::create(output).context("Failed to create output file")?;
        let header = VgmHeader::new(sid_clock, sid_model, 0);
        let mut vgm_writer =
            VgmWriter::new(vgm_file, header).context("Failed to write VGM header")?;

        let final_sample = (total_cycles * target_sample_rate / clock_rate) as u32;
        let mut loop_byte_offset = 0;
        let mut loop_sample_count = 0;
        let target_loop_sample = loop_point.map(|s| (s * target_sample_rate as f64) as u32);

        let mut current_sample = 0;
        for (sample, cmd) in vgm_events {
            if sample > current_sample {
                vgm_writer.write_command(&VgmCommand::WaitSamples(sample - current_sample))?;
                current_sample = sample;
            }

            // Capture the loop start after the bridging wait, so the byte offset
            // and the sample count both refer to the same stream position.
            if let Some(tls) = target_loop_sample
                && loop_byte_offset == 0
                && sample >= tls
            {
                loop_byte_offset = vgm_writer.current_pos()? as u32;
                loop_sample_count = final_sample - sample;
            }

            vgm_writer.write_command(&cmd)?;
        }

        if final_sample > current_sample {
            vgm_writer.write_command(&VgmCommand::WaitSamples(final_sample - current_sample))?;
        }

        vgm_writer
            .finalize(final_sample, loop_byte_offset, loop_sample_count)
            .context("Failed to finalize VGM")?;

        Ok(ConversionStats {
            event_count,
            total_samples: final_sample,
        })
    }
}
