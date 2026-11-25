# KindProbe Handoff

## [已完成] (Completed)
- **Phase 18 Performance Optimizations**:
  - `src/detectors/zip_ooxml/cd_check.rs`: Replaced unconditional dual `HashSet` heap allocations with `FastNameSet` (inline linear `Vec` for `<= 32` entries, automatic promotion to `HashSet` above 32).
  - `src/detectors/zip_ooxml/mod.rs`: Extracted shared `read_limited_xml_into_buf` and `finalize_ooxml_main_part` helpers (removed duplicated XML verification logic); retained `ZipArchive::new` after benchmarking proved direct local-header deflation regressed small OOXML latency.
  - `src/detectors/text/gateway.rs`: Added safe-Rust 8-byte SWAR ASCII / NUL / control-byte fast scan (`as_chunks::<8>()`) with shared `scan_ascii_slice` fallback.
- **Anti-Overfitting Benchmark & Binary Archival**:
  - Created `tests/test_phase18_ooxml_and_text_bench.rs` covering diverse OOXML (Deflated/Stored, custom targets, 6..58 entries), non-OOXML ZIP short-circuits, and 8K–16K ASCII/UTF-8/JSON/Rust/near-miss-binary payloads.
  - Archived pre-Phase 18 and Phase 18 release binaries in `target/phase_binaries/` (`kindprobe_{pre_phase18,phase18}.exe`, `test_performance_bench_{pre_phase18,phase18}.exe`, `test_anti_overfitting_bench_{pre_phase18,phase18}.exe`, `test_phase18_ooxml_and_text_bench_phase18.exe`).

## [当前状态] (Current State)
- `cargo clippy --all-targets --all-features -- -D warnings`: **0 warnings**.
- `cargo test --release --all-targets`: **All 55 test binaries passed** (including `test_phase18_ooxml_and_text_bench`).
- Note on debug run: `test_codex_acceptance_regression` debug exe experienced `os error 32` (locked by Kaspersky AV), while its Release counterpart passed in `cargo test --release --all-targets`.
- Side-by-side Phase 18 Release speedup:
  - `test_anti_overfitting_bench`: Mixed-Stream +28.3% (`951.37 ns` -> `741.34 ns`); Java `-34.8%`, C++ `-32.2%`, Verilog `-42.6%`, Rust `-26.0%`.
  - `test_performance_bench`: Plain ZIP `0.82 μs` -> `0.71 μs` (+15.7%), DOCX `26.03 μs` -> `20.12 μs` (+29.4%).

## [待办与阻塞] (Blockers & Next Steps)
- Working tree has uncommitted Phase 18 changes (`src/detectors/zip_ooxml/cd_check.rs`, `src/detectors/zip_ooxml/mod.rs`, `src/detectors/text/gateway.rs`, `tests/test_phase18_ooxml_and_text_bench.rs`, `.agents/HANDOFF.md`). Awaiting user confirmation before `git commit` / `git push`.
