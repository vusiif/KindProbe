# KindProbe Handoff

## [已完成] (Completed)
- **Round 5 (`c745c9e`)**: Committed and pushed NEF/ARW/DNG RAW, ONNX, C#, Swift, R, .env, and .gitignore.
- **Round 6 (LNK, MXF, SLN, CSPROJ, LaTeX, RST, AsciiDoc, Gradle, Nginx)**:
  - `src/detectors/signatures.rs`: Added Windows Shortcut **`lnk`** (`0x4C` + MS-SHLLINK CLSID) and SMPTE **`mxf`** (`06 0E 2B 34 ...` Partition Pack).
  - `src/detectors/text/xml_html.rs`: Added MSBuild **`csproj`** (`<Project Sdk=...>` / `<PropertyGroup>`).
  - `src/detectors/text/source_code.rs`: Added **`sln`** (VS Solution), **`tex`** (LaTeX), **`rst`** (reStructuredText), **`adoc`** (AsciiDoc), **`gradle`**, and **`nginx`**.
  - `src/extension/registry.rs`: Expanded sorted registry to **1,079** entries.
  - `tests/test_modern_formats_expansion.rs`: Added `test_round6_lnk_mxf_sln_csproj_tex_rst_adoc_gradle_nginx`.

## [当前状态] (Current State)
- Verification running via `cargo clippy` + `cargo test --release --all-targets`.

## [待办与阻塞] (Blockers & Next Steps)
- Working tree has uncommitted Round 6 changes. Awaiting user confirmation before `git commit` / `git push`.
