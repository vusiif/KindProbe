# KindProbe Handoff

## [已完成] (Completed)
- **Modern High-Frequency Format Expansion**:
  - `src/detectors/media/mp4.rs`: Added `avif` (`avif`/`avis`), `heic` (`heic`/`heix`/`hevc`/`hevx`/`heim`/`heis`), and `heif` (`mif1`/`msf1`/`heif`) ISO-BMFF image brands in both `_core` and `Report` paths.
  - `src/detectors/signatures.rs`: Added `probe_modern_binary_core` for `gguf` (`GGUF` v1..=3), `pst` (`!BDN`, also `.ost`), `tflite` (`TFL3` at offset 4), and `safetensors` (8B LE JSON header len + JSON tensor schema keys).
  - `src/detectors/zip_ooxml/{cd_check.rs, mod.rs, packages.rs}`: Added Android App Bundle `aab` (`BundleConfig.pb` in ZIP).
  - `src/detectors/text/{config.rs, source_code.rs}`: Added `eml` (RFC 5322 headers using UTF-8-safe byte slice checks), `ini` (`;` comment + `[section]`/`key=val`), `dockerfile` (`FROM ...` + instructions), and `css` (braces + CSS properties).
  - `src/extension/registry.rs`: Registered all 13 new extensions (`1019` total sorted entries).
  - `tests/test_modern_formats_expansion.rs`: Added dual-path & registry tests for all newly added formats.

## [当前状态] (Current State)
- `cargo clippy --all-targets --all-features -- -D warnings`: **0 warnings**.
- `cargo test --release --all-targets`: **All 59 test binaries passed**.

## [待办与阻塞] (Blockers & Next Steps)
- Working tree has uncommitted modern format expansion changes. Awaiting user confirmation before `git commit` / `git push`.
