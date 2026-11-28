# KindProbe Handoff

## [已完成] (Completed)
- Committed shared parser refactoring & index invariant test (`6354bea` on `main`).
- **Batch 5 — Deduplicate Generated Rules & Rebuild Static Index**:
  - Removed 14 unreachable exact-duplicate `SignatureRule` definitions across `images.rs` (`bpg`), `disk_images.rs` (`cso`), `roms.rs` (`ips`, `gba`), `models.rs` (`ply`), `fonts.rs` (`bdf`, `pcf`), `media.rs` (`ac3`, `tta`), `netpbm.rs` (`pam`, `xbm`), `vector_graphics.rs` (`qoi`), `scientific.rs` (`root`), and `multimedia_stream.rs` (`s3m`), reducing total declarative+generated rules cleanly from `790` to `776`.
  - Regenerated `src/detectors/rules/static_index.rs` (`BUCKETS`, `OFFSET_GROUPS`, `BY_FORMAT`) matching the pruned rule tables.
  - Strengthened `test_static_rule_index_invariants` in `tests/test_api_consistency.rs` to assert bucket+offset count parity and zero duplicate `(format_id, patterns)` entries.

## [当前状态] (Current State)
- Pending final full CI gate (`fmt`, `clippy`, `doc`, `--all-targets`) and auto-commit.

## [待办与阻塞] (Blockers & Next Steps)
- Optional future expansion: add real toolchain OOXML/ELF/PE/Wasm binary fixtures under `fixtures/real/` and `Cargo.toml` package metadata if publishing to crates.io.
