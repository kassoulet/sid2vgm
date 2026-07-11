---
project_name: 'sid2vgm-gemini'
user_name: 'Gautier'
date: '2026-06-12'
sections_completed: ['technology_stack', 'code_organization', 'language_idioms', 'vgm_format_invariants', 'timing_conversion', 'duration_loading', 'testing_fidelity', 'workflow_style', 'known_sharp_edges', 'out_of_scope']
status: 'complete'
optimized_for_llm: true
---

# Project Context for AI Agents

_Critical, non-obvious rules for implementing code in this repo. Read before touching `vgm/` or `convert.rs` — the format and timing invariants here are easy to break silently. Claims are tied to specific files so they stay verifiable._

> **Discovery:** this file lives at `docs/project-context.md` (the configured `project_knowledge` path). Claude Code does **not** auto-load it — there is no `CLAUDE.md` (intentionally removed). Read it on demand, or add a one-line pointer from a future `CLAUDE.md` if you want auto-loading.

---

## Technology Stack & Versions

- **Rust edition 2024** — Cargo workspace, resolver 2. Two **binary** crates (no library crate):
  - `sid2vgm/` — SID → VGM/VGZ converter (the main tool)
  - `sidvgmplay/` — VGM/VGZ → WAV renderer (verification + standalone playback)
- **CLI**: `clap 4.5` (derive macros) · **Errors**: `anyhow 1.0` · **Binary I/O**: `byteorder 1.5` · **Compression**: `flate2 1.0` (gzip; VGZ is gzipped VGM) · **Hashing**: `md5 0.7` · **WAV I/O**: `hound 3.5`

### The two-emulator split (critical, non-obvious)

- `sidplayfp-sys 0.1` — **cycle-accurate** C64 emulator. Used **only in `sid2vgm`** to *capture* SID register writes with exact cycle timestamps.
- `sidlite-sys 0.1` — lightweight SID. Used in **`sidvgmplay`** and in **`sid2vgm`'s dev-dependencies** (fidelity tests) to *render/verify* audio.
- These are not interchangeable. Capture = sidplayfp; render = sidlite.

### System requirement

- `libsidplayfp` development headers must be installed to build `sid2vgm` (and to run the fidelity tests, which build the sidplayfp reference).

## Critical Implementation Rules

### Code Organization

- **Module layout is domain-grouped; files are single-purpose.** Keep concerns separated by directory; don't fold `vgm/` encoding into `sid/` or vice versa. (Don't chase a line-count target — `renderer.rs` (~172), `tests/common.rs` (~160), and `convert.rs` (~125) are legitimately the largest files; size is not the metric, cohesion is.)
  - `sid/` — emulation & loading: `player.rs` (sidplayfp wrapper), `loader.rs` (PSID/RSID header parsing).
  - `trace/` — emulation tracing: `event.rs` (`SidEvent`).
  - `vgm/` — format encoding: `header.rs`, `commands.rs`, `writer.rs`, `compression.rs`.
  - Top-level: `cli.rs` (clap args), `convert.rs` (orchestration), `songlengths.rs`, `main.rs`.
- **Dependency direction**: `convert.rs` orchestrates `sid/` → `vgm/`. The *only* allowed cross-edge is `SidModel` (defined in `sid/mod.rs`, consumed by `vgm/header.rs`). Don't invert this layering — `vgm/` must not depend on `sid/` emulation.

### Language Idioms

- **Endianness is explicit and mixed** (highest-value rule): all multi-byte **VGM** fields are **little-endian** (`byteorder::LittleEndian`); **SID/PSID header** fields are **big-endian** (`u16::from_be_bytes`) — specifically `startSong` at offset `0x10` and the model/flags word at `0x76` in `sid/loader.rs`. Never assume native endianness.
- **Error handling**: return `anyhow::Result<T>`. Add `.context("...")` to every fallible I/O / FFI call. Wrap raw FFI/string errors with `map_err(|e| anyhow!(e))`.
- **Defensive, non-panicking parsing**: header/loader functions validate length and magic bytes (`PSID`/`RSID`) and **fall back to a sane default** rather than erroring — e.g. `read_sid_model` → `MOS6581`, `read_start_song` → `1`. Preserve this "tolerant reader" pattern.
- **Prefer let-else and if-let-chains** (edition 2024) for early returns — e.g. `let Ok(data) = std::fs::read(path) else { return SidModel::MOS6581 };` in `sid/loader.rs`.
- **`unwrap()`/`expect()` policy**: forbidden in `sid2vgm` production code (`cli.rs`, `convert.rs`, `sid/`, `vgm/`, `songlengths.rs`). Use `?`/`.context()`, or the `unwrap_or*` family in `main.rs` for genuinely-infallible defaults. Permitted in **test code** (`tests/*.rs`, `tests/common.rs`) and the **render hot loop** (`sidvgmplay/src/renderer.rs`), which unwrap intentionally.

### VGM Format Invariants (most error-prone — read before touching `vgm/`)

- **This is a NON-STANDARD VGM extension, not mainline VGM.** See `VGM_SID_SPEC.md` (v0.1). In the official VGM spec, `0xB6` is the µPD7759 command and offsets `0x78`/`0x7C` belong to the AY8910 — this repo repurposes them for SID. **Standard VGM players will not play these files correctly** (see the README compatibility note). Only `sidvgmplay` or a SID-aware player will. Don't "fix" the output to match the official spec.
- **Header is fixed 0x80 bytes**, data starts at `0x80`. Field `0x34` stores the data offset *relative to 0x34* (so it writes `0x80 - 0x34 = 0x4C`). Any reader must compute `data_start = 0x34 + read_u32(0x34)`, never hardcode `0x80`.
- **Back-patched fields** are written as zero up front and seeked-back in `VgmWriter::finalize`:
  - `0x04` EOF offset = `eof_pos - 4` (relative to 0x04).
  - `0x18` total samples (absolute sample count).
  - `0x1C` loop offset, written as `loop_offset - 0x1C` (relative to its own field). Only patched when `loop_offset > 0`.
  - `0x20` loop sample count.
  - Each relative offset is relative to its own field position — do not mix these up.
- **SID-specific header fields**: chip clock (u32) at `0x78` — a value of **0 means "SID not used"** per the spec, so `0` is invalid for a real tune and should be rejected/guarded by readers. Model byte at `0x7C`: **bit 0** selects model (`0`=MOS6581, `1`=MOS8580), bits 1–7 reserved — mask with `& 1`, don't compare the whole byte. Version pinned at `0x171` (VGM 1.71).
- **SID write command** is **`0xB6 <chip> <reg> <val>`** (4 bytes); `chip` selects SID 0/1 for multi-SID tunes. Valid SID registers are `0x00..=0x1F` — the renderer ignores writes with `reg > 0x1F` (`renderer.rs`); preserve that guard.
- **Wait-command encoding** (`commands.rs`) must be preserved exactly: `0x7n` = wait `n+1` (1–16 samples); `0x62` = 735 (1/60s); `0x63` = 882 (1/50s); `0x61 <u16le>` = arbitrary wait, **split into ≤0xFFFF chunks** for longer waits. `0x66` = end of data.
- **`finalize` must seek back to EOF** after patching so the stream position is correct — don't leave the cursor mid-header.
- The VGZ output is just the finished VGM gzipped (`flate2`, `Compression::best()`); the magic/header live *inside* the gzip, not alongside.

### Timing & Conversion Semantics

- **Clock rates are exact integers**: PAL = `985248` Hz, NTSC = `1022727` Hz (`Clock` enum). Target audio is always **44100** Hz. Never round or approximate these.
- **Cycle → sample conversion** is `sample = cycle * 44100 / clock_rate`, computed in **`u64`** with multiply-before-divide (matches `convert.rs`). **Overflow caveat:** the operands are attacker/CLI-controlled via `--duration`; `total_cycles * 44100` can overflow `u64` and `final_sample` can exceed `u32` for absurd durations. Guard the duration upper bound and/or use `checked_*` / `u32::try_from` rather than blindly casting `as u32`. Don't refactor to float (precision loss), but don't ignore overflow either.
- **PAL/NTSC selection**: `--pal`/`--ntsc` override the SID's auto-detected timing (`player.is_pal()`). The two flags are mutually exclusive (clap `conflicts_with`). Default = auto-detect, no override.
- **Capture runs in `CHUNK_CYCLES` (1_000_000) steps** up to `total_cycles = duration_secs * clock_rate`, accumulating events with absolute cycle timestamps.
- **Drop events past the capture window**: filter `e.cycle < total_cycles` before encoding. The emulator may run slightly past the requested count; un-dropped events would push waits beyond the header's `total_samples`.
- **Events are sorted by sample** before writing, then emitted as `WaitSamples(delta)` + `SidWrite`. A trailing wait pads up to `final_sample` so playback length matches the header exactly.
- **Loop point** (`--loop-point` seconds): capture the loop byte offset **after** the bridging wait is written, and only once (`loop_byte_offset == 0` guard); `loop_sample_count = final_sample - sample`. **Edge cases that are not yet guarded:** a loop point past the song end never matches → no loop is written (silent no-op); a negative value casts to sample 0 → loop at the start. Validate `0.0 <= loop_point < duration` if you touch this.

### Song Duration & SID Loading

- **Duration always comes from `Songlengths.txt` by default — never cap at 60s.** Hard project rule. `--duration` is the only override; absent it, look up the real length.
- **Songlengths lookup order**: (1) MD5 hash of the *entire SID file bytes* (HVSC format, lowercased hex key); (2) fall back to filename match. Within an entry, index by subtune; if the subtune index is missing, fall back to the first duration. **Caveat:** the hash only matches HVSC-canonical files byte-for-byte. A re-tagged or re-saved SID silently misses the hash path and falls through to the filename match — don't assume the hash always hits.
- **`Songlengths.txt` discovery** walks *up* from the input file's directory, checking both `<dir>/Songlengths.txt` and `<dir>/DOCUMENTS/Songlengths.txt` at each level (HVSC layout). If none found and no `--duration`, error out with an actionable message — don't silently pick a default length.
- **Subtune semantics**: subtune is **1-based**. `--subtune 0` means "the SID's default start song" — resolve it via `read_start_song` (the `startSong` field at header `0x10`, 1-based, min 1), not literally song 0.
- **Multi-SID files** are real: the `chip` field on each write selects SID 0/1. Preserve it end-to-end (`SidEvent.chip` → `VgmCommand::SidWrite.chip` → `0xB6 <chip>`). Don't collapse to a single chip.

### Testing & Fidelity

- **Round-trip fidelity is the core correctness test**: convert SID → VGM, render VGM → WAV via `sidlite`, and compare against a `sidplayfp` reference WAV by **RMS error**. The threshold is `rms_error < 0.5` (`tests/common.rs::compare_wavs`) — don't loosen it to make a test pass.
- **Shared test helpers live in `tests/common.rs`** (`render_vgm_to_wav`, `compare_wavs`). Reuse them; don't re-implement VGM parsing per test.
- **The test renderer must honor header fields**, not assume defaults: read SID model from `0x7C` (mask `& 1`), clock from `0x78`, and data start from `0x34` (relative). A renderer that hardcodes these will silently pass wrong output.
- **`batch_fidelity_test`** runs across the bundled test SIDs; the trimmed `Songlengths.txt` is committed to match exactly those files. If you add/remove test SIDs, update that file in lockstep.
- **Reproducing the suite**: `cargo test` must pass fully before review (per CONTRIBUTING.md), but it is **not** hermetic — it requires `libsidplayfp` dev headers installed and the bundled trimmed `Songlengths.txt`. A failure on a fresh/CI checkout is usually a missing dependency, not a code regression.
- **Robustness tests** (`sidvgmplay/src/security_test.rs`) feed malformed/truncated VGM to the renderer. The renderer must never panic on bad input: truncated commands surface as `Err` via `?`, unknown command bytes are skipped (`_ => {}`), and out-of-range registers (`reg > 0x1F`) are ignored. Preserve these three behaviors when editing `renderer.rs`.

### Development Workflow & Style

- **Conventional Commits** are mandatory (`feat:`, `fix:`, `refactor:`, `style:`, `docs:`, etc.) — terse, imperative, lowercase subject (see git history).
- **Do NOT add `Co-Authored-By` trailers** to commits.
- **`cargo fmt` and `cargo clippy` must be clean** before submitting — zero clippy warnings (enforced via pre-commit hooks). Run `cargo fmt` after any edit.
- **Intermediate-file cleanup**: VGZ conversion writes a `.vgm.tmp` then gzips it; always remove the temp file even on failure (see the error-path cleanup in `main.rs`). Any new intermediate artifact must follow the same always-cleanup discipline.
- **CLI output style** (`main.rs`): aligned `Label:    value` lines, `--stats` gated behind the flag. Match this format when adding output.

## Known Sharp Edges (unguarded paths — fix if you touch them)

_From a path-tracing review; these are reachable from CLI input and currently lack explicit guards._

- `renderer.rs` (`secs * 44100`) and `convert.rs` (`total_cycles * 44100`, `final_sample as u32`): arithmetic overflow on a large `--duration`. Use `saturating_mul` / `checked_*` / `u32::try_from`.
- `convert.rs` / `main.rs`: `duration_secs == 0` (via `--duration 0` or a `0:00` Songlengths entry) produces a degenerate/empty VGM. Reject early.
- `convert.rs`: negative or past-end `--loop-point` (see Timing section).
- `songlengths.rs` `parse_duration`: `minutes * 60` can overflow on a malformed huge minute field — use `checked_mul`/`checked_add`.
- `renderer.rs` / `convert.rs`: a VGM `sid_clock` of `0` (spec's "SID not used") flows into `set_sampling_parameters(0, …)` and zero-cycle clocking — guard against it.

## Out of Scope (don't propose these)

- No library crate — both members are binaries; don't extract a shared lib without a reason.
- No async, no logging framework, no config file — the tools are synchronous CLIs configured purely by flags.
- No network or external services. `Songlengths.txt` is read from the local HVSC layout only.

---

## Usage Guidelines

**For AI Agents:** Read this before implementing. The VGM Format Invariants, Timing & Conversion, and Known Sharp Edges sections are highest-risk — re-read them before editing `vgm/` or `convert.rs`. When in doubt, prefer the more restrictive option.

**For Humans:** Keep it lean and sid2vgm-specific. Update when the stack, the SID/VGM extension, or the fidelity threshold changes. Remove rules that become obvious.

Last reviewed: 2026-06-12 · rule_count: ~52
