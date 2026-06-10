# VGM Extension Specification: MOS SID (6581/8580)

**Version:** 0.1  
**Proposed Command:** 0xB6  
**Target VGM Version:** 1.71+

## 1. Introduction

This document defines an extension to the VGM (Video Game Music) file format to support the MOS Technology 6581 and 8580 SID (Sound Interface Device) chips, commonly found in the Commodore 64.

The goal is to provide a standardized way to store cycle-accurate register writes to one or more SID chips within a VGM stream.

## 2. Header Additions

The following fields are added to the VGM header to support the SID chip.

| Offset | Size | Name | Description |
| :--- | :--- | :--- | :--- |
| 0x78 | 4 bytes | SID clock | The clock frequency of the SID chip in Hz (e.g., 985248 for PAL, 1022727 for NTSC). |
| 0x7C | 1 byte | SID model | Flags defining the SID model and revision. |
| 0x7D | 3 bytes | Reserved | Set to 0. |

### 2.1 SID Clock (Offset 0x78)
A value of 0 indicates the SID chip is not used. A non-zero value specifies the clock frequency. For multiple SIDs, they are assumed to share the same clock unless otherwise specified in future revisions.

### 2.2 SID Model (Offset 0x7C)
The SID model byte defines the chip characteristics:
- `Bit 0`: Model Type (0 = MOS 6581, 1 = MOS 8580)
- `Bits 1-7`: Reserved for specific revisions (e.g., 6581R2, 8580R5)

## 3. Data Commands

The SID uses a 4-byte command for register writes.

### 3.1 Command 0xB6 - SID Write
`0xB6 cc rr vv`

- **0xB6**: Command identifier.
- **cc**: Chip index (0x00 for the 1st SID, 0x01 for the 2nd, etc.).
- **rr**: Register index (0x00 to 0x1F).
- **vv**: Value to write.

## 4. Register Mapping

Registers `0x00` through `0x1F` correspond to the standard SID memory map starting at base address `$D400`.

| Register Index | SID Function |
| :--- | :--- |
| 0x00 - 0x06 | Voice 1 (Freq, Pulse Width, Control, Attack/Decay, Sustain/Release) |
| 0x07 - 0x0D | Voice 2 |
| 0x0E - 0x14 | Voice 3 |
| 0x15 - 0x18 | Filter and Volume |
| 0x19 - 0x1A | Paddle X/Y (Read-only, writes ignored) |
| 0x1B - 0x1C | Voice 3 Waveform/ADSR (Read-only, writes ignored) |
| 0x1D - 0x1F | Unused/Reserved |

## 5. Timing

VGM timing remains sample-based at 44100 Hz. SID register writes should be placed at the nearest sample position corresponding to their original CPU cycle.

```text
sample = floor(cpu_cycle * 44100 / sid_clock)
```

## 6. Sample Implementation (Rust)

```rust
fn write_sid_command<W: Write>(writer: &mut W, chip: u8, reg: u8, val: u8) -> io::Result<()> {
    writer.write_all(&[0xB6, chip, reg, val])
}
```
