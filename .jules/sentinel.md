## 2026-06-11 - [MEDIUM] Missing Input Validation in SID Register Writes
**Vulnerability:** The VGM renderer (`sidvgmplay`) was vulnerable to crashes (Denial of Service) when processing malformed or malicious VGM files containing out-of-bounds SID register indices (0x20-0xFF).
**Learning:** Rust's memory safety prevents memory corruption, but out-of-bounds access on array-backed data structures (common in hardware emulators) still causes a panic, leading to DoS.
**Prevention:** Always validate external inputs that act as indices into hardware registers or fixed-size buffers, even when using memory-safe languages.
