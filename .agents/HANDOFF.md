# KindProbe Handoff

## [已完成] (Completed)
- **Phase 17 Security Hardening**:
  - `src/detectors/zip_ooxml/{opc.rs, xml.rs, mod.rs}`: Converted `parse_package_relationships`, `parse_content_types`, and `verify_main_part_xml` to generic `R: BufRead` streaming APIs; `mod.rs` now wraps `DecompressLimitReader` in `BufReader::with_capacity(4096, ...)` and streams XML events directly without allocating a 256 KiB decompression buffer.
  - `src/detectors/cfb/ppt.rs`: Added `[MS-PPT]` `RecordHeader` validation (`recType` in `0x03E8..=0x3000`, non-zero `recLen` for non-container records) and `CurrentUserAtom` validation (`recType == 0x0FF6`, `headerToken == 0xE391_C05F | 0xF3D1_C4DF`). Updated test fixtures in `tests/test_cfb_extended.rs` and `tests/test_anti_overfitting_bench.rs`.
  - `src/detectors/zip_ooxml/cd_check.rs`: Added ZIP local file header & compressed payload out-of-bounds / CD overlap guard (`lfh_off >= cd_offset` or `lfh_off + 30 + fn_len + compressed_size > cd_offset`).
  - `src/budget.rs`: Added `io::ErrorKind::Interrupted` retry loops in `BudgetReader::read` and `DecompressLimitReader::read`.
  - `tests/test_phase17_security_hardening.rs`: Added 4 regression tests covering `EINTR` retry, streaming XML budget exhaustion, ZIP overlap rejection, and forged PPT `Current User` atom rejection.
  - Archived `*_pre_phase17.exe` and `*_phase17.exe` binaries in `target/phase_binaries/`.

## [当前状态] (Current State)
- `cargo clippy --all-targets --all-features -- -D warnings`: **0 warnings**.
- `cargo test --release --all-targets`: **All 56 test binaries passed** (including `test_phase17_security_hardening` and `test_phase18_ooxml_and_text_bench`).
- Side-by-side Performance Parity (`pre_phase17` vs `phase17`):
  - `test_anti_overfitting_bench`: Transitional DOCX `20.06 μs` -> `19.75 μs` (`50,631 docs/s`), MS-PPT `39.89 μs` -> `33.26 μs` (`30,064 docs/s`).
  - `test_phase18_ooxml_and_text_bench`: TOTAL `0.110 s` -> `0.116 s` (OOXML `28,047` -> `25,528 ops/s` due to `BufReader` + `&mut dyn BufRead` streaming trade-off for 0-256 KiB peak memory).

## [待办与阻塞] (Blockers & Next Steps)
- Working tree has uncommitted Phase 17 changes (`src/budget.rs`, `src/detectors/cfb/ppt.rs`, `src/detectors/zip_ooxml/{cd_check.rs,mod.rs,opc.rs,xml.rs}`, `tests/{test_anti_overfitting_bench.rs,test_cfb_extended.rs,test_phase17_security_hardening.rs}`, `.agents/HANDOFF.md`). Awaiting user confirmation before `git commit` / `git push`.
