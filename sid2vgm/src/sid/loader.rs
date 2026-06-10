use crate::sid::SidModel;
use std::path::Path;

/// Extract SID chip model from a PSID/RSID file header.
/// Defaults to MOS6581 for v1 files or when the flags are ambiguous.
pub fn read_sid_model(path: &Path) -> SidModel {
    let Ok(data) = std::fs::read(path) else {
        return SidModel::MOS6581;
    };
    if data.len() < 6 || (&data[0..4] != b"PSID" && &data[0..4] != b"RSID") {
        return SidModel::MOS6581;
    }
    let version = u16::from_be_bytes([data[4], data[5]]);
    // Flags field at 0x76 only exists in PSID v2+
    if version < 2 || data.len() < 0x78 {
        return SidModel::MOS6581;
    }
    // Flags bits 4-5: SID1 model — 00=unknown, 01=6581, 10=8580, 11=both
    let flags = u16::from_be_bytes([data[0x76], data[0x77]]);
    match (flags >> 4) & 0x3 {
        0b10 => SidModel::MOS8580,
        _ => SidModel::MOS6581,
    }
}
