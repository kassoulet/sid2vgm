# sid2vgm

Convert Commodore 64 SID music files to VGM format.

## Why

The [VGM](https://vgmrips.net/wiki/VGM_Specification) (Video Game Music) format is a sample-accurate log of hardware register writes. It captures the exact sequence of chip commands needed to reproduce a piece of music — which means a VGM player only needs to replay those writes; it doesn't need to run a full CPU emulator.

The C64's SID chip has a rich ecosystem of emulators and players, but the canonical format (PSID/RSID) bundles the original 6510 machine code that drives the chip. Playing a SID file requires a complete C64 system emulator. This is powerful but heavy — and incompatible with VGM-based players and hardware.

`sid2vgm` bridges the gap: it runs a cycle-accurate C64 emulator ([libsidplayfp](https://github.com/libsidplayfp/libsidplayfp)) under the hood, intercepts every SID register write with its exact cycle timestamp, and encodes those writes into a VGM stream. The result plays on any VGM-aware player or hardware without a C64 emulator.

This is useful for:
- Playing C64 music on VGM-based hardware (FPGA boards, dedicated players)
- Archiving SID music in a chip-agnostic format
- Feeding SID music into VGM-based tools (visualizers, analyzers, editors)

## Tools

This workspace contains two binaries:

- **`sid2vgm`** — converts a `.sid` file to `.vgm` or `.vgz`
- **`sidvgmplay`** — renders a `.vgm` or `.vgz` file to `.wav`

## Building

Requires Rust (2024 edition) and `libsidplayfp` development headers (see [CONTRIBUTING.md](CONTRIBUTING.md)).

```bash
cargo build --release
```

Binaries are placed in `target/release/`.

## Usage

### sid2vgm

```
sid2vgm <input.sid> [options]

Options:
  -o, --output <file>       Output file (.vgm or .vgz). Default: input name with .vgm extension.
  -s, --subtune <num>       Subtune number (default: 0 = default start song from SID header)
  -d, --duration <secs>     Capture duration in seconds (default: from Songlengths.txt lookup; errors if absent)
  --loop-point <secs>       Mark a loop point in the VGM stream
  --pal / --ntsc            Force PAL or NTSC clock (default: from SID header)
  --stats                   Print event count, duration, and file size
```

Example:

```bash
sid2vgm Andropolis.sid -o Andropolis.vgm -d 372
sid2vgm Andropolis.sid -o Andropolis.vgz          # gzip-compressed output
```

### sidvgmplay

```
sidvgmplay <input.vgm|.vgz> [options]

Options:
  -o, --output <file>       Output .wav file. Default: input name with .wav extension.
  -d, --duration <secs>     Limit output length in seconds (default: full duration)
```

Example:

```bash
sidvgmplay Andropolis.vgm -o Andropolis.wav
sidvgmplay Andropolis.vgz                    # decompresses transparently
```

## VGM format details

The output targets VGM v1.71 with the SID chip extension:

- SID register writes use command `0xB6 <chip> <reg> <val>`
- SID clock frequency at header offset `0x78`
- SID chip model byte at header offset `0x7C` (`0` = MOS 6581, `1` = MOS 8580)
- Wait commands: `0x62` (735 samples, NTSC 1/60s frame), `0x63` (882 samples, PAL 1/50s frame), `0x61 nn nn` (arbitrary), `0x7n` (1–16 samples)

See [VGM_SID_SPEC.md](VGM_SID_SPEC.md) for the full specification.

## License

MIT — see [LICENSE](LICENSE).

## Author

Gautier Portet — [kassoulet@gmail.com](mailto:kassoulet@gmail.com)
