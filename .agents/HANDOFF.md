# KindProbe Handoff

## [已完成] (Completed)
- **Round 3 (`0c26968`)**: Committed and pushed AppImage, PHP, Proto, SQL, Lua, Julia, Nix, TSX, JSX, and Snap.
- **Round 4 (DMG, UTF-16 BOM Text, REG, PowerShell & Release Workflow)**:
  - `src/detectors/mod.rs`: Added macOS **`dmg`** (`koly` + v4 512-byte trailer) across both slice (`probe_dmg_slice_core`) and stream (`probe_dmg_stream_core`) paths.
  - `src/detectors/text/mod.rs`: Added `probe_utf16_bom_text_core` supporting UTF-16 LE/BE BOM text (bounded 4 KiB decode, rejecting UTF-32 BOM, unpaired surrogates, NULs, and >1% control chars) and fallback `probe_source_code_core` under `[` dispatch.
  - `src/detectors/text/{config.rs, source_code.rs}`: Added Windows **`.reg`** (`Windows Registry Editor Version 5.00` / `REGEDIT4`) and **PowerShell `.ps1`** (`[CmdletBinding()]`/`param(`/`$PSScriptRoot`), and prevented TOML `[section]` from matching `[CmdletBinding()]`.
  - `src/extension/registry.rs`: Expanded sorted registry to **1,058** entries (`dmg`, `ps1`, `psd1`, `psm1`, `reg`).
  - `.github/workflows/release.yml`: Added multi-platform Release build & SHA256 packaging workflow triggered on `v*` tags.
  - `tests/test_modern_formats_expansion.rs`: Added `test_round4_dmg_utf16_reg_and_powershell`.

## [当前状态] (Current State)
- `cargo clippy --all-targets --all-features -- -D warnings`: **0 warnings**.
- `cargo test --release --all-targets`: **All 59 test binaries passed**.

## [待办与阻塞] (Blockers & Next Steps)
- Working tree has uncommitted Round 4 changes. Awaiting user confirmation before `git commit` / `git push`.
