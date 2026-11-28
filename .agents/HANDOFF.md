# KindProbe Handoff

## [已完成] (Completed)
- Pushed `432ea71`, `a418dc9`, and `3221d19` cleanly to `origin/main`.
- **Batch 3 — Additional Short-ASCII Rules & OOXML / Matroska Refactor**:
  - Added structural validators `validate_binary_6_from_4`, `validate_nmea`, `validate_kml_open`, and `validate_gerber` in `src/detectors/rules/engine.rs` and wired them onto `nmea` & `kml` (`cad_and_gnss.rs`), `bik` (`game_assets.rs`), `gxt` (`geospatial_vector_cad.rs`), and `gbr` (`robotics_firmware.rs`).
  - Added false-positive guards in `tests/test_text_extended.rs` (`$GPS_MODE`, `BIKING`, `GXT`, `%FS`).
  - Extracted `src/detectors/zip_ooxml/direct_reader.rs` from `zip_ooxml/mod.rs` (shrinking `mod.rs` to ~575 lines) and eliminated `.to_ascii_lowercase()` heap allocation for `.xml` suffix checks.
  - Deduplicated `probe_matroska` in `src/detectors/media/matroska.rs` to delegate to `probe_matroska_core`.

## [当前状态] (Current State)
- Pending final full-suite `cargo test --all-targets` verification after Batch 3 changes.

## [待办与阻塞] (Blockers & Next Steps)
- Optionally deduplicate duplicate generated rule entries (`bpg`, `cso`, `ips`, `bdf`, `tta`, `pcf`, `qoi`, `root`, `s3m`, `cas`/`msx_rom_cas`, `uasset`/`unreal_uasset`) + regenerate `static_index.rs`, and expand real OOXML/ELF/PE fixtures.
