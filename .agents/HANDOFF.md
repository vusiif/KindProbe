# KindProbe Handoff

## [已完成] (Completed)
- **Previous Batch (`31c8a93`)**: Committed and pushed AVIF/HEIC/HEIF/GGUF/Safetensors/TFLite/PST/AAB/EML/INI/Dockerfile/CSS.
- **Active Format Expansion (Round 2)**:
  - `src/detectors/media/mp4.rs`: Added `m4b`, `3gp`, `3g2`, and Canon `cr3` ISO-BMFF brands.
  - `src/detectors/signatures.rs`: Added `raf` (`FUJIFILMCCD-RAW `), `cr2` (TIFF + `CR` at offset 8), `npy` (`\x93NUMPY`), `asar` (Chromium Pickle + `{"files":{`), and `ova` (TAR `ustar` with `.ovf` first entry).
  - `src/detectors/zip_ooxml/{cd_check.rs, mod.rs}`: Added `npz` (`*.npy` inside ZIP).
  - `src/detectors/text/{xml_html.rs, source_code.rs}`: Added `vue`, `svelte`, `cmake`, `makefile`, `hcl`/`tf`, `graphql`, `zig` (before Rust `pub fn main(` loop), `dart`, `kt`, `scala` (before Python `def`), and `scss` (before CSS).
  - `src/extension/registry.rs`: Expanded sorted registry to **1,047** entries (including `pkg` -> `xar`).
  - `tests/test_modern_formats_expansion.rs`: Added dual-path assertions for all new formats.

## [当前状态] (Current State)
- `cargo clippy --all-targets --all-features -- -D warnings`: **0 warnings**.
- `cargo test --release --all-targets`: **All 59 test binaries passed**.

## [待办与阻塞] (Blockers & Next Steps)
- Working tree has uncommitted Round 2 active format changes. Awaiting user confirmation before `git commit` / `git push`.
