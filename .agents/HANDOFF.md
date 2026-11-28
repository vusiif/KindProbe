# KindProbe Handoff

## [已完成] (Completed)
- Fixed `-j` batch execution (`src/cli/batch.rs`) with true concurrent worker threads, bounded in-flight window/work queue, streaming directory walk preserving sorted DFS order, immediate first-line JSONL flushing, and explicit traversal/stdout write error handling, plus barrier/order/write-failure tests (`src/cli/batch.rs`, `tests/test_phase19_batch_cli.rs`).
- Aligned slice and reader budget/probe semantics and clamped stream header buffer allocation to `initial_window_size.min(max_source_bytes_read)` (`src/detectors/mod.rs`, `src/detectors/disks/iso.rs`), with budget boundary parity tests (`tests/test_api_consistency.rs`).
- Added MS-CFB pre-parse directory chain and entry budget enforcement prior to `CompoundFile::open` (`src/detectors/cfb/mod.rs`, `tests/test_budget_adversarial.rs`) and documented exact budget scopes in `README.md`.
- Unified structural `AndroidManifest.xml` (AXML & XML) validation across ZIP fast path and `ZipArchive` fallback (`src/detectors/zip_ooxml/packages.rs`, `src/detectors/zip_ooxml/direct_reader.rs`, `src/detectors/zip_ooxml/cd_check.rs`, `src/detectors/zip_ooxml/mod.rs`), covering DEX-less valid APKs and rejecting `classes.dex`-only or spoofed manifests (`tests/test_packages_and_audio_extended.rs`).

## [当前状态] (Current State)
- All 5 audit findings addressed; awaiting final verification + auto-commit on `main`.

## [待办与阻塞] (Blockers & Next Steps)
- Push to remote or cut next patch release tag when requested.
