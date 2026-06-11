use crate::sid::SidModel;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Result, Write};

#[derive(Debug, Clone, Copy)]
pub enum Clock {
    Pal = 985248,
    Ntsc = 1022727,
}

pub struct VgmHeader {
    pub version: u32,
    pub sid_clock: u32,
    pub sid_model: SidModel,
    pub data_offset: u32,
    pub eof_offset: u32,
    pub total_samples: u32,
}

impl VgmHeader {
    pub fn new(clock: Clock, sid_model: SidModel, total_samples: u32) -> Self {
        Self {
            version: 0x00000171, // VGM 1.71
            sid_clock: clock as u32,
            sid_model,
            data_offset: 0x80, // data starts at 0x80
            eof_offset: 0,     // back-patched by VgmWriter::finalize
            total_samples,
        }
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(b"Vgm ")?; // 0x00
        writer.write_u32::<LittleEndian>(self.eof_offset)?; // 0x04 — back-patched
        writer.write_u32::<LittleEndian>(self.version)?; // 0x08
        writer.write_u32::<LittleEndian>(0)?; // 0x0C SN76489 clock
        writer.write_u32::<LittleEndian>(0)?; // 0x10 YM2413 clock
        writer.write_u32::<LittleEndian>(0)?; // 0x14 GD3 offset
        writer.write_u32::<LittleEndian>(self.total_samples)?; // 0x18
        writer.write_u32::<LittleEndian>(0)?; // 0x1C loop offset
        writer.write_u32::<LittleEndian>(0)?; // 0x20 loop samples
        writer.write_u32::<LittleEndian>(0)?; // 0x24 rate
        writer.write_u16::<LittleEndian>(0)?; // 0x28 SN76489 feedback
        writer.write_u8(0)?; // 0x2A SN76489 shift
        writer.write_u8(0)?; // 0x2B SN76489 flags
        writer.write_u32::<LittleEndian>(0)?; // 0x2C YM2612 clock
        writer.write_u32::<LittleEndian>(0)?; // 0x30 YM2151 clock
        writer.write_u32::<LittleEndian>(self.data_offset - 0x34)?; // 0x34 data offset (relative)

        // Pad 0x38–0x77 with zeros (unused chip clocks)
        for _ in 0..(0x78 - 0x38) {
            writer.write_u8(0)?;
        }

        writer.write_u32::<LittleEndian>(self.sid_clock)?; // 0x78 SID clock
        let model_byte: u8 = match self.sid_model {
            // 0x7C SID model
            SidModel::MOS6581 => 0,
            SidModel::MOS8580 => 1,
        };
        writer.write_u8(model_byte)?;
        writer.write_u8(0)?; // 0x7D reserved
        writer.write_u8(0)?; // 0x7E reserved
        writer.write_u8(0)?; // 0x7F reserved

        Ok(())
    }
}
