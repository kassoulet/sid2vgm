use crate::vgm::commands::VgmCommand;
use crate::vgm::header::VgmHeader;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Result, Seek, SeekFrom, Write};

pub struct VgmWriter<W: Write + Seek> {
    writer: W,
    #[allow(dead_code)]
    header: VgmHeader,
}

impl<W: Write + Seek> VgmWriter<W> {
    pub fn new(mut writer: W, header: VgmHeader) -> Result<Self> {
        header.write(&mut writer)?;
        Ok(Self { writer, header })
    }

    pub fn write_command(&mut self, command: &VgmCommand) -> Result<()> {
        command.write(&mut self.writer)
    }

    pub fn current_pos(&mut self) -> Result<u64> {
        self.writer.stream_position()
    }

    pub fn finalize(
        &mut self,
        total_samples: u32,
        loop_offset: u32,
        loop_samples: u32,
    ) -> Result<()> {
        self.write_command(&VgmCommand::EndData)?;

        let eof_pos = self.writer.stream_position()?;
        let eof_offset = (eof_pos - 4) as u32;

        // Update EOF offset
        self.writer.seek(SeekFrom::Start(4))?;
        self.writer.write_u32::<LittleEndian>(eof_offset)?;

        // Update Total Samples (offset 0x18)
        self.writer.seek(SeekFrom::Start(0x18))?;
        self.writer.write_u32::<LittleEndian>(total_samples)?;

        if loop_offset > 0 {
            // Update Loop offset (offset 0x1C)
            self.writer.seek(SeekFrom::Start(0x1C))?;
            self.writer.write_u32::<LittleEndian>(loop_offset - 0x1C)?;

            // Update Loop samples (offset 0x20)
            self.writer.seek(SeekFrom::Start(0x20))?;
            self.writer.write_u32::<LittleEndian>(loop_samples)?;
        }

        // Seek back to end
        self.writer.seek(SeekFrom::Start(eof_pos))?;
        Ok(())
    }
}
