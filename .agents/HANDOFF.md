# KindProbe Handoff

## [已完成] (Completed)
- Pushed `432ea71`, `a418dc9`, `3221d19` and committed `1074218` (`nmea`/`kml`/`bik`/`gxt`/`gbr` hardening, `direct_reader.rs`, `matroska.rs` deduplication).
- **Batch 4 — Shared Header Parsers & Static Rule Index Invariants**:
  - Consolidated `probe_mp4` onto `probe_mp4_core` (`src/detectors/media/mp4.rs`, removing ~180 duplicate lines).
  - Extracted shared parsing helpers in `src/detectors/executables/pe.rs` (`valid_pe_header_offset`), `src/detectors/executables/elf.rs` (`parse_elf_meta`), `src/detectors/executables/macho.rs` (Fat Mach-O branch deduplication), and `src/detectors/media/riff.rs` (`parse_riff_subtype`).
  - Added `test_static_rule_index_invariants` to `tests/test_api_consistency.rs` enforcing sorted `BY_FORMAT`, ascending `OFFSET_GROUPS`, and complete rule coverage.

## [当前状态] (Current State)
- Pending final verification gate (`fmt`, `clippy`, `doc`, `--all-targets`) and auto-commit on `main`.

## [待办与阻塞] (Blockers & Next Steps)
- Optional polish: deduplicate remaining `GENERATED_RULES` redundant entries (if regenerating `static_index.rs`) and add real OOXML/ELF/PE binary fixtures under `fixtures/real/`.
