#[cfg(test)]
mod tests {
    use crate::renderer::render;
    use std::path::Path;
    use std::fs;

    #[test]
    fn test_invalid_sid_register() {
        let mut vgm = Vec::new();
        vgm.extend_from_slice(b"Vgm "); // Magic
        vgm.extend_from_slice(&0u32.to_le_bytes()); // EOF offset (dummy)
        vgm.extend_from_slice(&0x00000171u32.to_le_bytes()); // Version
        vgm.extend_from_slice(&0u32.to_le_bytes()); // SN76489 clock
        vgm.extend_from_slice(&0u32.to_le_bytes()); // YM2413 clock
        vgm.extend_from_slice(&0u32.to_le_bytes()); // GD3 offset
        vgm.extend_from_slice(&100u32.to_le_bytes()); // Total samples
        vgm.extend_from_slice(&0u32.to_le_bytes()); // Loop offset
        vgm.extend_from_slice(&0u32.to_le_bytes()); // Loop samples
        vgm.extend_from_slice(&0u32.to_le_bytes()); // Rate
        vgm.extend_from_slice(&0u16.to_le_bytes()); // SN76489 feedback
        vgm.extend_from_slice(&[0u8, 0u8]); // SN76489 shift/flags
        vgm.extend_from_slice(&0u32.to_le_bytes()); // YM2612 clock
        vgm.extend_from_slice(&0u32.to_le_bytes()); // YM2151 clock
        vgm.extend_from_slice(&0x4Cu32.to_le_bytes()); // Data offset (relative to 0x34). 0x34 + 0x4C = 0x80

        // Pad to 0x78
        while vgm.len() < 0x78 {
            vgm.push(0);
        }
        vgm.extend_from_slice(&985248u32.to_le_bytes()); // SID clock
        vgm.push(0); // SID model
        vgm.extend_from_slice(&[0, 0, 0]); // Reserved

        // Pad to 0x80
        while vgm.len() < 0x80 {
            vgm.push(0);
        }

        // Data: 0xB6 command with invalid register 0xFF
        vgm.push(0xB6);
        vgm.push(0x00); // Chip 0
        vgm.push(0xFF); // Register 0xFF (INVALID)
        vgm.push(0x42); // Value
        vgm.push(0x66); // End of data

        let output_path = Path::new("security_test.wav");
        let result = render(&vgm, output_path, None);

        // Cleanup
        if output_path.exists() {
            let _ = fs::remove_file(output_path);
        }

        assert!(result.is_ok(), "Renderer should handle invalid register gracefully");
    }
}
