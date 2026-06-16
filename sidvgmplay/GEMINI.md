# sidvgmplay

`sidvgmplay` is a Rust-based command-line utility for rendering VGM (Video Game Music) and VGZ (compressed VGM) files that contain SID music into high-quality WAV audio files.

## Project Overview

- **Purpose**: Convert Commodore 64 SID music stored in VGM format to WAV.
- **Main Technologies**:
    - **Rust**: Core language (edition 2024).
    - **sidlite-sys**: Provides the SID chip emulation.
    - **hound**: Handles WAV file encoding.
    - **clap**: Used for command-line argument parsing.
    - **flate2**: Enables transparent support for `.vgz` (gzip-compressed) files.
    - **anyhow**: Standardized error management.

## Architecture

- `src/lib.rs`: Library entry point, exposing core logic, CLI arguments, and utility functions.
- `src/main.rs`: Binary entry point, handles CLI execution and file I/O using the library.
- `src/cli.rs`: Defines the command-line interface using `clap`.
- `src/renderer.rs`: The core logic that parses VGM commands and drives the SID emulation to produce audio samples.
- `src/security_test.rs`: Unit tests focusing on robustness and handling of invalid VGM data.
- `tests/regeneration.rs`: Integration test that renders all VGM/VGZ files in `tests/vgms/` to `tests/output/`.

## Building and Running

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (edition 2024 support required).

### Commands
- **Build**:
  ```bash
  cargo build --release
  ```
- **Run**:
  ```bash
  cargo run -- <input.vgm> [OPTIONS]
  ```
  Example:
  ```bash
  cargo run -- music.vgm -o music.wav -d 60
  ```
- **Test**:
  ```bash
  cargo test
  ```

## Development Conventions

- **Error Handling**: Use `anyhow::Result` for application-level errors and provide context with `.with_context()`.
- **Modularity**: Keep the rendering logic decoupled from the CLI and file I/O where possible.
- **Safety**: The renderer should gracefully handle malformed or malicious VGM files without crashing (see `security_test.rs`).
- **Performance**: High-performance emulation is handled by `sidlite-sys`. Avoid unnecessary allocations in the inner rendering loop (`clock_chips`).

## Usage Options

- `input`: (Required) Path to the `.vgm` or `.vgz` file.
- `-o, --output <path>`: Path to the output `.wav` file. Defaults to the input filename with a `.wav` extension.
- `-d, --duration <seconds>`: Limit the output to a specific duration. By default, it renders the entire file as specified in the VGM header.
