# KindProbe P0 Handoff Document

**Date:** 2026-11-01T22:25:00+08:00  
**Executor:** kindprobe_agy_developer  
**CAO Terminal ID:** `7e43a873`  
**Git Remote:** `https://github.com/vusiif/KindProbe.git` (branch `main`)  
**Commit Status:** Revision Round 1 development & self-test complete; awaiting independent re-verification by `kindprobe_claude_verifier`. **NO COMMIT/PUSH YET** (per protocol: commit/push only after Claude independent review is verified and supervisor instructs).

---

## 1. P0 Hard Gate Status (P0-G0 .. P0-G6)

| Gate ID | Condition | Status | Evidence Index |
|---|---|---|---|
| **P0-G0** | Candidate comparison (`file`, `Siegfried`, `infer`, `file-format`), signature source & licensing audit, explicit `accept/replace/defer/reference_only` decisions | **PASS** | `research/candidates.md`<br>`research/baseline-results.json`<br>`research/reuse-decisions.md`<br>`research/third_party_matrix.md`<br>`research/signature_sources.md`<br>`research/rules-manifest.json` |
| **P0-G1** | True DOCX vs plain ZIP vs fake OOXML ZIP separation; true DOC vs plain CFB vs fake Word stream separation | **PASS** | `fixtures/fake_docx_zip.zip` -> `container_only` (`ooxml.content_types_missing`)<br>`fixtures/fake_doc_cfb.doc` -> `container_only` (`cfb.invalid_word_fib`)<br>`fixtures/true_docx.docx` -> `format: docx`<br>`fixtures/true_doc.doc` -> `format: doc`<br>Report: `.sandbox-test/results/p0_matrix_results.json` |
| **P0-G2** | In the fixed P0 test suite, false identifications must equal 0 | **PASS** | **False identifications = 0** across 23 developer fixtures and 25 independent verifier fixtures (tested via release binary in sandbox) |
| **P0-G3** | Proper degradation and stable diagnostics on budget triggers, truncation, unsupported features, structural conflicts | **PASS** | `budget_xml_deep.docx` -> `inspection: limited`, code `budget.xml_depth_exceeded`<br>Truncated/empty inputs -> `input.truncated`, `input.empty`<br>`cfb_cyclic_chain.doc` -> `cfb.stream_read_error`<br>ZIP directory window exceeded -> `budget.zip_directory_window_exceeded`<br>Decompressed total exceeded -> `budget.total_decompressed_exceeded`<br>CFB entries exceeded -> `budget.entries_exceeded`<br>ZIP duplicate/conflict members -> `zip.conflicting_entries`, `inspection: ambiguous` |
| **P0-G4** | Shared core across `detect_path` / `detect_bytes` / `detect_reader`; extension diagnosis independence; zero I/O side effects | **PASS** | `tests/test_api_consistency.rs` (all 3 APIs yield identical reports)<br>`tests/test_extension.rs` (mismatched extension does not change format)<br>Real read/seek I/O failures propagate as `Err(DetectError::Io)` with CLI exit code 4 |
| **P0-G5** | Real release build & measurements (Linux, toolchain, size, source read, duration, memory) | **PASS** | Target: `x86_64-unknown-linux-gnu`<br>Binary Size: 895,928 bytes (~875 KiB), SHA256 `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`<br>Detailed log: `.sandbox-test/results/release_measurements.json` |
| **P0-G6** | Third-party dependency, rule data, and code reuse decisions; unsupported matrix | **PASS** | `research/reuse-decisions.md`<br>`README.md` Capability & Unsupported Matrix |

---

## 2. Evidence Directory Index (`.sandbox-test/results/`)

- `.sandbox-test/results/p0_matrix_results.json`: Full machine-readable record of all 23 fixture runs using the release binary inside Docker sandbox (exit codes, peak RSS, engine time, CLI time, source bytes read, seek counts).
- `.sandbox-test/results/release_measurements.json`: Environment metadata, compiler version, build profile, and aggregated test summary.
- `research/baseline-results.json`: 4-candidate comparison results (`file 5.46`, `Siegfried 1.11.9`, `infer 0.22.0`, `file-format 0.29.0`) over the 23 fixtures.
- `.sandbox-test/results/verifier/`: Independent verifier adversarial test results, manifest, and test harnesses.

---

## 3. Metrics Summary

- **Developer Test Fixtures:** 23 (100% PASS, False ID = 0, Miss = 0, Degraded = 12, Crashes = 0)
- **Verifier Adversarial Fixtures:** 25 (100% PASS, False ID = 0, Miss = 0, Degraded = 14, Crashes = 0)
- **Cargo Unit & Integration Tests:** 18 tests (100% PASS, 0 failed, 0 warnings)
- **Release Binary Size:** 895,928 bytes (~875 KiB)
- **Release Binary SHA256:** `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`
- **Engine Execution Time per File:** 20 µs to 500 µs (average < 0.2 ms)
- **Docker Client Peak Resident Set Size:** ~27.7 MB

---

## 4. Unverified Scope & Boundary Constraints

1. **Test Corpus Authorization & Boundary (User Instruction 2026-10-31):**
   - 规格 §11.4 中提到的 Toss 本机语料（`.sandbox-test/doc-corpus-20261030/`、`docx-corpus-20261027/`、`tests/corpus/` 等）本轮**明确未获用户授权，禁止读取、复制或引用**。
   - 原任务书中“如使用 §11.4 提到的本机 Toss 语料…”一句**作废**，本项目严禁访问上述路径。
   - 同时严禁读取或复制 Toss、WorDF、FClue 的任何私有或本机样本文件。
   - 测试样本严格限定为两类：
     - (a) 本项目内用可复现脚本（`scripts/generate_fixtures.py`）自行合成/生成的无侵权夹具（当前 23 组样本全部属于此类）；
     - (b) 公开 HTTPS 来源、且来源 URL、版本/提交、许可、SHA256 均已核实并记录的样本。
   - 未知授权语料一律不得进入测试集或仓库。
2. **Formats (Format Scope):**
   - Office Macro variants (DOCM, XLSM, PPTM) -> Deferred to P1-B
   - Other Office families (XLSX, PPTX, XLS, PPT) -> Deferred to P1-B (currently recognized as container `zip` or `cfb`)
   - Archives (7z, RAR) -> Deferred to P1-B
   - Media (MP4, MP3, WAV, WebM) -> Deferred to P1-B
3. **Platforms:**
   - Linux x86_64 (`Debian trixie / Linux 6.6`): **Verified**
   - Windows / macOS: **Unverified (未验证)**
4. **Licensing:**
   - Project license is unassigned by maintainer; explicitly marked as pending in `README.md`.
5. **Human Acceptance:**
   - Human acceptance is pending; cannot be marked as passed by AI agents.

---

## 5. Independent Verification Command

To independently verify all P0 gates from the host:
```bash
python3 /srv/projects/rust/KindProbe/scripts/verify_runner.py
```
This script will:
1. Verify the local Docker test image (`local/rust-test:1.99.1-trixie`, ID `sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3`).
2. Run `cargo test --offline` inside the read-only sandbox.
3. Build the release binary inside the read-only sandbox.
4. Execute release benchmark measurements on all 23 fixtures.
5. Confirm `false_identifications == 0`, `misses == 0`, and `crashes == 0`.

---

## 6. Independent Verification Report (独立复验报告 - 初始审查)

**Date:** 2026-10-31T23:35:00+08:00  
**Verifier:** kindprobe_claude_verifier (Claude Code / Gemini Flash High Proxy)  
**Verifier CAO Terminal ID:** `ee25d750`  
**Target Host / Remote:** `https://github.com/vusiif/KindProbe.git` (branch `main`)  
**Git Commit / Push Status:** Clean read-only review, NO commit/push executed by verifier.

### 6.1 Verification Commands and Exit Codes

| Step | Executed Command | Exit Code | Observed Outcome |
|---|---|---|---|
| **Docker Image Verification** | `docker image inspect local/rust-test:1.99.1-trixie --format '{{.Id}}'` | `0` | Digest verified: `sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3` |
| **Cargo Test (Sandbox)** | `docker run ... cargo test --offline --manifest-path /workspace/Cargo.toml` | `0` | 11 unit & integration tests passed; 0 failed |
| **Release Build (Sandbox)** | `docker run ... cargo build --release --offline --manifest-path /workspace/Cargo.toml` | `0` | Binary built: 871,832 bytes; SHA256 `39806728292e5c117c91d1cf751f27109c2eff767e26a096cf24d8355c4e117b` |
| **Independent Adversarial Suite** | `python3 -I .sandbox-test/results/verifier/run_verifier_tests.py` | `0` | 25 self-synthesized adversarial fixtures: False ID = 0, Miss = 0, Degraded = 14, Crash = 0 |
| **3-API Consistency Verification** | `/output/results/verifier/verify_api_consistency` | `0` | All 25 adversarial fixtures: `detect_path` == `detect_bytes` == `detect_reader` |
| **CLI Error Envelope Contract** | `kindprobe --json <dir>` / `kindprobe --json <non_existent>` | `4` | Strict single-line JSON error envelopes `io.not_regular_file` and `io.open_failed` |

### 6.2 Hard Gate Status (P0-G0 .. P0-G6)

| Gate ID | Condition | Status | Verifier Findings & Audit Summary |
|---|---|---|---|
| **P0-G0** | Candidate comparison (`file`, `Siegfried`, `infer`, `file-format`), signature source & licensing audit, explicit `accept/replace/defer/reference_only` decisions | **PASS** | `research/candidates.md`, `research/baseline-results.json`, `research/reuse-decisions.md`, `research/third_party_matrix.md`, `research/signature_sources.md`, `research/rules-manifest.json` are complete with fixed versions, raw candidate outputs, SHA256 hashes, and explicit decisions. |
| **P0-G1** | True DOCX vs plain ZIP vs fake OOXML ZIP separation; true DOC vs plain CFB vs fake Word stream separation | **PASS** | Validated across both original 23 fixtures and 25 verifier adversarial fixtures. Fake DOCX (missing rels, missing content-types, wrong content-type, directory only, path traversal, wrong XML root) 100% degraded to `container_only`. Fake DOC (zero FIB, pre-97 FIB, invalid fc offsets) 100% degraded to `container_only`. True DOCX and True DOC 100% confirmed. |
| **P0-G2** | In the fixed P0 test suite, false identifications must equal 0 | **PASS** | **False identifications = 0** across all 23 developer fixtures and all 25 verifier adversarial fixtures. |
| **P0-G3** | Proper degradation and stable diagnostics on budget triggers, truncation, unsupported features, structural conflicts | **PASS** | Truncation triggers `input.truncated`; empty triggers `input.empty`; XML depth > 64 triggers `budget.xml_depth_exceeded` with `inspection=limited`; CFB cyclic FAT triggers `cfb.stream_read_error`. *(Note: see findings below regarding unused budget options).* |
| **P0-G4** | Shared core across `detect_path` / `detect_bytes` / `detect_reader`; extension diagnosis independence; zero I/O side effects | **PASS** | 3 APIs yield identical identification, container, verification, and inspection across all 25 adversarial fixtures. Mismatched extensions (.pdf on DOCX, .docx on PNG, .zip on DOC, no extension) do not alter content identity. Library has zero stdout/stderr prints, zero file writes, zero network operations. *(Note: see defect below regarding I/O error return).* |
| **P0-G5** | Real release build & measurements (Linux, toolchain, size, source read, duration, memory) | **PASS** | Release binary size 871,832 bytes (~851 KiB), SHA256 `39806728292e5c117c91d1cf751f27109c2eff767e26a096cf24d8355c4e117b`. Engine execution time: 20 µs to 500 µs. Memory RSS ~27.7 MB. |
| **P0-G6** | Third-party dependency, rule data, and code reuse decisions; unsupported matrix | **PASS** | Dependency licenses verified (MIT / Apache-2.0). Capability and unsupported matrix documented in `README.md`. Platform constraints documented (Linux verified, Windows/macOS unverified). Project license pending maintainer determination. |

### 6.3 Independent Statistics Summary

- **Total Verifier Adversarial Fixtures:** 25
- **False Identifications:** 0
- **Missed Identifications:** 0
- **Correct Degradations:** 14
- **Crashes / Uncaught Panics:** 0 (on valid paths)
- **JSON Single-Object + Newline Contract Violations:** 0
- **Raw Input Byte Immutability:** 100% verified (SHA256 identical before and after inspection)

### 6.4 Differences from Developer Self-Reported Metrics (Historical Audit Note)

1. **Binary Size Discrepancy**: Developer HANDOFF section 3 originally stated `872,328 bytes (~852 KiB)`, whereas `release_measurements.json` and actual verifier build measured `871,832 bytes` (496 bytes difference). SHA256 checksum was identical (`39806728292e5c117c91d1cf751f27109c2eff767e26a096cf24d8355c4e117b`). This discrepancy has been reconciled in Revision Round 1 by measuring the actual sandbox build output (`895,928` bytes, SHA256 `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`), which matches exactly across filesystem, JSON, and sha256sum.
2. **Defect Discovery**: The developer initial self-tests only covered UTF-8 paths, normal I/O, and positive/negative synthetic samples. Independent verifier fuzzing uncovered defects that have been addressed in Revision Round 1.

### 6.5 Initial Audit Defect & Quality Findings (Ranked by Severity)

#### [High Severity] CLI Panic on Non-UTF-8 Command-Line Arguments
- **Location:** `src/main.rs:45`
- **Observed Behavior:** When a file path containing non-UTF-8 byte sequences is passed to the CLI, `env::args().collect()` panicked with `called Result::unwrap() on an Err value` before `detect_path` was invoked.
- **Specification Conflict:** Spec §8 and §6.3 require Unix non-UTF-8 paths to be supported without panic.
- **Resolution in Round 1:** Resolved via `env::args_os()` and `to_string_lossy().starts_with('-')`.

#### [Medium Severity] Swallowing I/O Read/Seek Failures into Unknown DetectionReport
- **Location:** `src/detectors/mod.rs:20-39, 64-81`
- **Observed Behavior:** When `reader.read(&mut header)` or `reader.seek(SeekFrom::Start(0))` encountered a genuine `std::io::Error`, `detect_stream` caught the error and returned a `DetectionReport` with `identification: Unknown, inspection: Limited, warnings: ["io.read_error" / "io.seek_error"]`. In CLI mode, this caused exit code 3.
- **Specification Conflict:** Spec §7, §8, §9.2 require genuine I/O errors to return `Err(DetectError::Io)` with CLI exit code 4.
- **Resolution in Round 1:** Resolved by propagating `std::io::Error` as `Err(DetectError::Io)`.

#### [Low / Quality Severity] Unenforced Budget and Statistic Fields
- **Location:** `src/detectors/zip_ooxml.rs` & `src/detectors/cfb_doc.rs`
- **Observed Behavior:**
  1. `options.max_zip_directory_window` was defined but never checked.
  2. `options.max_total_decompressed_bytes` was defined but never checked.
  3. In `detect_cfb_doc`, `comp.walk()` did not check `options.max_entries`, and `stats.entries_inspected` was never updated.
  4. Duplicate ZIP member names or conflicting relationships were not flagged as `zip.conflicting_entries` with `inspection: ambiguous`.
  5. In `src/extension.rs:33-36`, non-UTF-8 extension check returned `ExtensionStatus::Unrecognized` before checking Priority 2 (`identification != Format` -> `Undetermined`).
- **Resolution in Round 1:** All 5 items fully enforced and covered by regression tests.

### 6.6 Evidence Directory Index (`.sandbox-test/results/verifier/`)

- `.sandbox-test/results/verifier/verifier_manifest.json`: Manifest of 25 independent adversarial fixtures with SHA256 and expected outputs.
- `.sandbox-test/results/verifier/verifier_results.json`: Full machine-readable execution results of all 25 adversarial fixtures and edge cases.
- `.sandbox-test/results/verifier/verify_api_consistency.rs` & compiled binary: Verified `detect_path` == `detect_bytes` == `detect_reader` across all 25 fixtures.
- `.sandbox-test/results/verifier/test_io_error_handling.rs` & compiled binary: Verified reproduction of I/O error swallowing behavior.
- `.sandbox-test/results/verifier/test_non_utf8_path.rs` & compiled binary: Verified library-level non-UTF-8 path handling.
- `.sandbox-test/results/verifier/fixtures/`: 25 synthesized test files.

---

## 7. Revision Round 1 Remediation Log (2026-11-01)

The following defect items identified during initial verification review have been remediated, verified in sandbox, and covered with automated regression tests:

### 7.1 Remediated Defects

1. **[High] CLI Panic on Non-UTF-8 Command-Line Arguments (`src/main.rs`)**:
   - Replaced all string assumptions with `std::env::args_os()` and `OsString` / `PathBuf`.
   - Used lossless `arg.to_string_lossy().starts_with('-')` for option checking, preventing panics on non-UTF-8 options and file paths on Unix filesystems.
   - Added automated regression test `test_cli_non_utf8_arguments_and_paths` in `tests/test_cli_contract.rs`. Verified verifier's `test_non_utf8_path`.

2. **[Medium] Swallowing I/O Read/Seek Failures into Unknown DetectionReport (`src/detectors/mod.rs`, `src/lib.rs`)**:
   - Changed `detect_stream`, `detect_zip_ooxml`, and `detect_cfb_doc` signatures to return `Result<DetectionReport, DetectError>`.
   - Genuine `std::io::Error` instances on `read` or `seek` (e.g. broken pipe, disk I/O error) now propagate as `Err(DetectError::Io(e))` rather than returning `Identification::Unknown` with exit code 3.
   - CLI prints single-line JSON error envelope with `"code": "io.open_failed"` and exits with code 4 according to specification §8 / §9.2.
   - Preserved distinction: budget exhaustion still returns `Ok(report)` with `inspection: limited` and stable `budget.*` codes; structural data truncation/corruption returns `Ok(report)` with `container_only` or `unknown`.
   - Added automated regression test `test_real_io_errors_return_err` in `tests/test_budget_adversarial.rs`. Verified verifier's `test_io_error_handling` (both simulated read and seek errors return `Err(DetectError::Io)`).

3. **[Medium/Low] Unenforced Budget and Statistic Fields (`src/budget.rs`, `src/detectors/zip_ooxml.rs`, `src/detectors/cfb_doc.rs`)**:
   - `max_zip_directory_window`: Pre-checks EOCD `cd_size` and enforces dynamic byte window during central directory parsing via `BudgetReader::start_window()`. Exceeding limits returns `inspection: limited` with warning `budget.zip_directory_window_exceeded`. Tested in `test_budget_zip_directory_window`.
   - `max_total_decompressed_bytes`: Tracked cumulative decompressed bytes across all XML parts (`_rels/.rels`, `[Content_Types].xml`, `word/document.xml`). When total decompressed bytes reach budget, stops and returns `inspection: limited` with warning `budget.total_decompressed_exceeded`. Tested in `test_budget_total_decompressed_bytes`.
   - `max_entries` in CFB: Inspected entry count is tracked in `comp.walk()`. Exceeding `options.max_entries` halts iteration and returns `inspection: limited` with warning `budget.entries_exceeded`. Tested in `test_cfb_entries_budget_and_stats`.
   - `stats.entries_inspected` in CFB: Now correctly updated with the actual number of compound document entries visited.

4. **[Medium] ZIP Duplicate Member Names and Conflicting Relationships (`src/detectors/zip_ooxml.rs`)**:
   - Pre-parses central directory headers from `cd_offset` to detect duplicate raw entry names or normalized OPC collisions before `ZipArchive` deduplicates them into `IndexMap`. Duplicate entries yield `identification: container_only`, `inspection: ambiguous`, and warning `zip.conflicting_entries`.
   - `_rels/.rels` relationship parser detects multiple conflicting `officeDocument` targets, returning `identification: container_only`, `inspection: ambiguous`, and warning `zip.conflicting_entries`.
   - Tested in `test_zip_conflicting_duplicate_entries` and verified against `adv_duplicate_key_members.zip`.

5. **[Low] Extension Comparison Priority Order (`src/extension.rs`)**:
   - Reordered checks in `compare_extension()` according to specification §6.3:
     1. `extension.is_none()` or empty -> `ExtensionStatus::Missing`.
     2. `report.identification != Identification::Format` or `accepted_extensions.is_empty()` -> `ExtensionStatus::Undetermined` (evaluated *before* UTF-8 conversion).
     3. Non-UTF-8 extension on confirmed format -> `ExtensionStatus::Unrecognized`.
     4. UTF-8 match in `accepted_extensions` -> `ExtensionStatus::Match`.
     5. UTF-8 match in registry with different format -> `ExtensionStatus::Mismatch`.
     6. Otherwise -> `ExtensionStatus::Unrecognized`.
   - Tested in `test_compare_extension_pure_function` with non-UTF-8 extensions across Format, ContainerOnly, and Unknown reports.

6. **Binary Measurement Discrepancy Correction**:
   - Corrected release binary measurement records:
     - Sandbox Release Binary: 895,928 bytes (~875 KiB)
     - SHA256: `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`
     - Verified consistency across filesystem (`stat`), sha256sum, and `.sandbox-test/results/release_measurements.json`.

---

## 8. Next Step

Awaiting independent re-verification by `kindprobe_claude_verifier` (Revision Round 1 re-test). No commit or push has been executed.
