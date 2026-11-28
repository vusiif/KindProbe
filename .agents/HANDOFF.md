# KindProbe Handoff

## [已完成] (Completed)
- Committed initial security/correctness fixes (`2963321`) and repo-wide `cargo fmt` + CI `fmt --check` / `test --doc` enforcement (`130b281`).
- **Remaining Hardening & Engineering Cleanup**:
  - Wired all 29 fixtures from `fixtures/verifier_manifest.json` directly into `tests/test_codex_acceptance_regression.rs` (`test_verifier_manifest_29_adversarial_matrix`).
  - Fixed text/short-signature false positives (`source_code.rs` Markdown with `#include`, `config.rs` `[TOC]` vs `toml` & Markdown frontmatter vs `yaml`, `xml_html.rs` generic `<tag>...</tag>` fallback, `telecom_exchange.rs` SREC hex validator, `textures.rs` Crunch binary header validator).
  - Made `scripts/run_52_fixtures_regression.py` and `scripts/run_acceptance.py` Windows-safe (`getuid`/`getgid` guard), and guarded deprecated `tools/codegen.py` behind `--force`.

## [当前状态] (Current State)
- Running final `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo test --doc`.

## [待办与阻塞] (Blockers & Next Steps)
- Optional long-term refactor: modularize `src/detectors/text/source_code.rs` (3,000 LOC after formatting) into token-based classifiers and deduplicate remaining legacy non-`_core` wrappers.
