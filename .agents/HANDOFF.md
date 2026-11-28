# KindProbe Handoff

## [已完成] (Completed)
- Fixed batch worker-panic deadlock (`src/cli/batch.rs`) by wrapping worker task execution in `catch_unwind` + `TaskPanicGuard`/`QueueCloseGuard`, emitting terminal `batch.worker_panic` records for failed sequence numbers, closing queues/windows immediately, and returning exit code `1` without hanging (`test_batch_worker_panic_on_first_of_80_does_not_hang`).
- Fixed Reader DMG header-boundary EOF miss (`src/detectors/mod.rs`) via `probe_dmg_header_at_exact_eof` verifying exact stream end without overflowing `max_source_bytes_read`, plus 4,096-byte DMG consistency tests (`tests/test_api_consistency.rs`).
- Tightened AXML structural validation (`src/detectors/zip_ooxml/packages.rs`) to parse `ResStringPool` UTF-8/UTF-16 strings and verify an actual balanced `<manifest>` root `RES_XML_START_ELEMENT_TYPE` node with `package`/`xmlns:android`, while keeping the public 2-arg `check_zip_packages` signature backwards-compatible (`check_zip_packages_with_options` for caller-supplied options) and covering negative pool-only AXML + compat tests (`tests/test_packages_and_audio_extended.rs`).

## [当前状态] (Current State)
- Verification pending on final edits; will auto-commit once checks pass.

## [待办与阻塞] (Blockers & Next Steps)
- Push to remote when requested by user.
