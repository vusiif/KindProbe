# Changelog

## [0.2.0] - 2026-11-26

### Performance
- **Zero-Allocation Core Engine (`DetectionCore`)**: Stack-only `Copy` detection path via `detect_bytes_core`, `detect_reader_core`, and `detect_path_core`.
- **Static Read-Only Lookup & Bucketed Extension Registry**: Compile-time `.rodata` bucket tables (`u32`/`u64` aligned checks) and binary-search extension registry (1006+ extensions).
- **SWAR Text Gateway**: 8-byte SWAR ASCII/NUL/control-byte fast scan (`>= 64B`) with zero-overhead scalar fallback (`< 64B`).
- **ZIP Central Directory Dedup**: Inline linear `FastNameSet` for small archives (`<= 32` entries) avoiding `HashSet` heap allocations.
- **Multi-Core Parallel Batch CLI**: `--batch`, `-r/--recursive`, `--jsonl`, and `-j/--jobs <N>` powered by `std::thread::scope` and `detect_path_core`.

### Security
- **Streaming OOXML XML Parsing**: `BufReader<DecompressLimitReader>` streaming pull-parsing for `_rels/.rels`, `[Content_Types].xml`, and main document parts (eliminates 256 KiB decompression buffer peaks).
- **Structural Anti-Spoofing**: `[MS-PPT]` `RecordHeader` & `CurrentUserAtom` validation (`0x0FF6`, `0xE391C05F`/`0xF3D1C4DF`) and ZIP Central Directory overlap/OOB local header rejection.
- **Resilience & Fuzzing**: `EINTR` (`ErrorKind::Interrupted`) retry loops in `BudgetReader` and `DecompressLimitReader`, standalone `cargo-fuzz` targets (`fuzz/`), differential fuzz regression tests, and RustSec CI audit (`taiki-e/install-action@cargo-audit` + `cargo audit`).

## [0.1.0] - 2026-11-23
- Initial release with content-first detection engine, bounded `BudgetReader`, JSON v1 CLI contract, and 1000+ format rules.
