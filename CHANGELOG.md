# Changelog

## [0.2.0] - 2026-11-27

### Performance
- **Zero-Allocation Core Engine (`DetectionCore`)**: Stack-only `Copy` detection path via `detect_bytes_core`, `detect_reader_core`, and `detect_path_core`.
- **Single-Pass OOXML Central Directory Reuse**: Cached verified Stored/Deflate OOXML entry offsets (`direct_reader.rs`) during initial Central Directory walk, skipping duplicate EOCD/CD parsing with safe `ZipArchive` fallback for edge cases.
- **Static Read-Only Lookup & Bucketed Extension Registry**: Compile-time `.rodata` bucket tables (`776` deduplicated rules across `static_index.rs`) and binary-search extension registry (`1,399` mappings).
- **SWAR Text Gateway & Modular Source Detector**: 8-byte SWAR ASCII/NUL/control-byte fast scan (`>= 64B`) and modularized source/config/XML heuristics (`source_code/` submodules).
- **ZIP Central Directory Dedup & Core Parity**: Inline linear `FastNameSet` for small archives (`<= 32` entries) and shared metadata parsers across binary/media/archive detectors.
- **Multi-Core Parallel Batch CLI**: `--batch`, `-r/--recursive`, `--jsonl`, and `-j/--jobs <N>` powered by `std::thread::scope`, modularized under `src/cli/`.

### Security & Hardening
- **Streaming OOXML XML Parsing & Traversal Defense**: `BufReader<DecompressLimitReader>` streaming pull-parsing for `_rels/.rels`, `[Content_Types].xml`, and main document parts; strict OPC path traversal & conflicting entry checks; `BudgetReader` EOF/seek hardening.
- **Short Signature & Text Heuristic Disambiguation**: Structural/non-ASCII validators for short & ASCII-prefix rules (`ac3`/`eac3`, `pk`/`pkf`, `3ds`, `eot`, `der`, `srec`, `crn`, `xbm`, `stl`, `orc`, `ihex`, `smk`, `sp3`, `nmea`, `kml`, `bik`, `gxt`, `gbr`) and refined Markdown/TOML/YAML/XML detection.
- **Structural Anti-Spoofing & Invariants**: `[MS-PPT]` `RecordHeader` & `CurrentUserAtom` validation (`0x0FF6`, `0xE391C05F`/`0xF3D1C4DF`), ZIP Central Directory overlap/OOB local header rejection, and compile-time rule index invariant checks.
- **Resilience, Fuzzing & Verification Matrix**: `EINTR` retry loops in `BudgetReader`/`DecompressLimitReader`, `verifier_manifest.json` integration, `37` real-world/spec-compliant binary fixtures (`fixtures/real/`), runnable `lib.rs` doctests, `cargo-fuzz` targets (`fuzz/`), and RustSec CI audit.

### Packaging & Tooling
- Enforced `cargo fmt --check` and `cargo test --doc` in CI, added crates.io package metadata to `Cargo.toml`, and hardened Python helper scripts for cross-platform execution.

## [0.1.0] - 2026-11-23
- Initial release with content-first detection engine, bounded `BudgetReader`, JSON v1 CLI contract, and 1000+ format rules.
