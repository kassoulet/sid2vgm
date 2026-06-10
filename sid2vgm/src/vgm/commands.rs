use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Result, Write};

pub enum VgmCommand {
    SidWrite { chip: u8, reg: u8, val: u8 },
    WaitSamples(u32),
    EndData,
}

impl VgmCommand {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<()> {
        match self {
            VgmCommand::SidWrite { chip, reg, val } => {
                writer.write_u8(0xB6)?;
                writer.write_u8(*chip)?;
                writer.write_u8(*reg)?;
                writer.write_u8(*val)?;
            }
            VgmCommand::WaitSamples(samples) => {
                let mut remaining = *samples;
                while remaining > 0 {
                    if remaining <= 16 {
                        // 0x7n: wait n+1 samples
                        writer.write_u8(0x70 | ((remaining - 1) as u8))?;
                        remaining = 0;
                    } else if remaining == 735 {
                        writer.write_u8(0x62)?;
                        remaining = 0;
                    } else if remaining == 882 {
                        writer.write_u8(0x63)?;
                        remaining = 0;
                    } else {
                        let wait = if remaining > 0xFFFF {
                            0xFFFF
                        } else {
                            remaining as u16
                        };
                        writer.write_u8(0x61)?;
                        writer.write_u16::<LittleEndian>(wait)?;
                        remaining -= wait as u32;
                    }
                }
            }
            VgmCommand::EndData => {
                writer.write_u8(0x66)?;
            }
        }
        Ok(())
    }
}
