use crate::trace::event::SidEvent;
use anyhow::{Result, anyhow};
use sidplayfp_sys::Player as SidPlayer;
use std::path::Path;

pub struct Player {
    inner: SidPlayer,
    total_cycles: u64,
}

impl Player {
    pub fn new() -> Result<Self> {
        let mut inner = SidPlayer::new().map_err(|e| anyhow!(e))?;
        inner.set_roms(None, None, None);
        Ok(Self {
            inner,
            total_cycles: 0,
        })
    }

    pub fn load_file(&mut self, path: &Path, subtune: u16) -> Result<()> {
        self.inner.load_file(path, subtune).map_err(|e| anyhow!(e))
    }

    pub fn step(&mut self, cycles: u32) -> Result<Vec<SidEvent>> {
        let elapsed = self.inner.play(cycles).map_err(|e| anyhow!(e))?;
        let events = self
            .inner
            .get_writes()
            .iter()
            .map(|w| SidEvent {
                cycle: self.total_cycles + w.cycle as u64,
                chip: w.sid_num,
                register: w.reg,
                value: w.val,
            })
            .collect();
        self.total_cycles += elapsed as u64;
        Ok(events)
    }

    pub fn is_pal(&self) -> bool {
        self.inner.is_pal()
    }
}
