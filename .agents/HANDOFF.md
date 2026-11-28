# KindProbe Handoff

## [已完成] (Completed)
- **OPC & ZIP Security Hardening**: Rejected root-escaping `..` traversal in `try_normalize_opc_path` (`src/detectors/zip_ooxml/opc.rs`), rejected multiple conflicting `officeDocument` relationships (`ooxml.conflicting_relationships`), fixed basename extension extraction in `parse_content_types`, unblocked OPC `.vsix`/`.nupkg` packages (`src/detectors/zip_ooxml/mod.rs`), validated EOCD comment length and real parsed CD entry count/truncation (`cd_check.rs`), and added `Event::Empty` XML depth enforcement (`xml.rs`).
- **Content-First & Budget Consistency**: Restricted `fast_path` (`src/detectors/fast_path.rs`) so only byte-0 `%PDF-` short-circuits and text extension hints cannot preempt binary/ISO/declarative rules; added natural-EOF probe and configurable `max_seek_count` to `BudgetReader` (`src/budget.rs`, `src/lib.rs`); aligned ISO/DMG slice budget and stats (`src/detectors/mod.rs`).
- **Core/Report Parity & Registry Completeness**: Aligned `macho.rs` (including Java `--enable-preview` `minor == 0xFFFF`), `ttf_otf.rs`, `nested.rs`, and `DetectionCore::from_report` (`src/model.rs` all 53 categories + container evidence/warnings); registered all 20 missing declarative rule extensions (`19` generated + `udeb`) in `src/extension/registry.rs`.
- **Short-Input, Media & CLI Fixes**: Tightened 1-byte `Truncated` heuristics (`jpeg`, `gif`, `7z`, `zstd`, `pcap`, `vmdk`, `pdf`, and rule `engine.rs` offset-0 matches; preserved `b"M"` -> `pe` truncation contract) and required `>= 2` bytes for plain text confirmation (`text/mod.rs`), fixed `ftyp` 64-bit extended `box_size == 1` (`mp4.rs`), removed dead boolean clause (`source_code.rs`), fixed `--batch` directory symlink skipping, worker panic accounting, `-j 0` validation, and Chinese localization (`src/main.rs`), plus updated `README.md` and `CHANGELOG.md`.

## [当前状态] (Current State)
- Full `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `fixtures/verifier_manifest.json` re-verification pending final run below.

## [待办与阻塞] (Blockers & Next Steps)
- Remaining follow-up items (deferred): modularize `src/detectors/text/source_code.rs` (2700+ LOC substring chain), remove/rebuild outdated `tools/codegen.py`, run repo-wide `cargo fmt` as a dedicated formatting commit, and decide whether to update the 2 legacy `expected_container: null` entries in `fixtures/verifier_manifest.json` (`adv_truncated_zip_8bytes.zip`, `adv_truncated_cfb_16bytes.doc`) to match current `container_only` behavior.
