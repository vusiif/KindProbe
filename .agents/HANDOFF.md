# KindProbe Handoff

## [已完成] (Completed)
- Committed short-rule hardening, OOXML single-pass CD reuse, core probe deduplication, `source_code` modularization, and `src/lib.rs` doctests (`432ea71`).
- Committed 3-byte / pure-ASCII rule hardening (`xbm`, `stl`, `orc`, `ihex`, `smk`, `sp3`) and CLI modularization into `src/cli/{mod,i18n,batch}.rs` (`a418dc9`).

## [当前状态] (Current State)
- Clean baseline on `main` (verified `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --doc`, and `cargo test --all-targets`).

## [待办与阻塞] (Blockers & Next Steps)
- Potential next optimizations:
  1. Harden remaining 3-4 byte ASCII rules without validators (`nmea` `b"$GP"`, `bik` `b"BIK"`, `gxt` `b"GXT"`, `gbr` `b"%FS"`, `sp3` / `kml` `b"<kml"`) and deduplicate redundant generated rule entries (`bpg`, `cso`, `ips`, `bdf`, `tta`, `pcf`, `qoi`, `root`, `s3m`, `cas`/`msx_rom_cas`, `uasset`/`unreal_uasset`) while regenerating `static_index.rs`.
  2. Split `src/detectors/zip_ooxml/mod.rs` (1,015 lines) by extracting direct LFH streaming into `direct_reader.rs`, and eliminate remaining structural duplication between `probe_*` and `probe_*_core` (`probe_signatures`, `probe_riff`, `probe_mp4`, `probe_elf`, `probe_pe`, `probe_macho`).
  3. Expand `fixtures/real/` coverage beyond the current 25 fixtures (OOXML/ELF/PE/Wasm/MP3/MP4/WebP) and populate `Cargo.toml` publishing metadata (`repository`, `readme`, `keywords`, `categories`).
