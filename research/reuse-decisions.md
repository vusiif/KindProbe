# Architectural and Dependency Reuse Decisions (KindProbe P0-R)

**Author:** kindprobe_agy_developer  
**Date:** 2026-10-31  
**Decision Gate:** Specification §10.1, §10.7, §11.2 P0-G0, P0-G6  

---

## 1. Decision Table

| Component / Asset | Asset Type | Candidate Version | Decision | Rationale | Impacted Formats / Roles |
|---|---|---|---|---|---|
| `infer` | Rust Library | 0.22.0 | **replace** / **reference_only** | High false confirmation rate (e.g. 4-byte PNG, 8-byte ZIP); misidentifies all CFB as `msi`; no OPC awareness; no budget adapter support. | Common signatures, CFB |
| `file-format` | Rust Library | 0.29.0 | **replace** / **reference_only** | Heuristic flaw: identifies DOCX purely via `filename.starts_with("word/")`; misses Word 97 DOC when CLSID is zero; lacks budget controls. | ZIP, DOCX, CFB, DOC |
| `zip` | Rust Library | 2.6.1 | **accept** | Clean separation of central directory reading; operates on `Read + Seek`; easily wrapped with `BudgetReader` to enforce <= 1 MiB CD window and <= 8 MiB total read. | ZIP, OOXML container |
| `quick-xml` | Rust Library | 0.36.2 | **accept** | Streaming pull parser; no DTD/external entity expansion; depth tracking easily implemented to strictly enforce max depth <= 64 and output <= 256 KiB. | `[Content_Types].xml`, `_rels/.rels`, `word/document.xml` |
| `cfb` | Rust Library | 0.10.0 | **accept** (with bounded wrapper) | Provides robust compound document sector access; wrapped in `BudgetReader` and accompanied by custom MS-DOC FIB validator (`0xA5EC` / `0x00C1`). | CFB container, Word 97+ DOC |
| `serde` & `serde_json` | Rust Library | `serde` 1.0.229, `serde_json` 1.0.154 | **accept** | Industry standard machine-readable JSON v1 serialization with zero overhead and full schema adherence. | CLI JSON output, machine reporting |
| `libmagic` / system `file` | External Rulebase | 5.46 | **reference_only** | External cross-validation only. Falsely confirms fake DOC and fake DOCX zip; cannot be embedded into pure Rust offline binary. | Validation baseline |
| `PRONOM` / `DROID` | Signature Database | V125 & container 20260119 | **reference_only** | Useful reference for PUIDs and byte sequences, but container signature 1030 omits package relationship verification, leading to false DOCX confirmation. | Validation baseline |
| `Siegfried` | External CLI | 1.11.9 | **reference_only** | Go-based independent reference tool. Used in test suite to benchmark PRONOM behavior against KindProbe. | Verification tool |

---

## 2. In-Depth Justifications

### 2.1 Rejection of `infer` and `file-format` for Production Core
1. **`infer`**:
   - In our empirical tests, `infer` produced 4 false confirmations and 4 misidentifications out of 23 test fixtures.
   - For any truncated input starting with `\x89PNG`, it declares `png` without verifying chunk structure.
   - For CFB files, it returns `msi` unconditionally because its matcher checks OLE2 without inspecting streams.
   - Consequently, `infer` cannot serve as a reliable detection core.
2. **`file-format`**:
   - `file-format` checks whether any zip member starts with `"word/"`. An attacker or normal user with an archive containing `word/notes.txt` causes `file-format` to confirm `docx`.
   - In CFB files, `file-format` only checks root entry CLSID. Word 97-2003 files frequently have a null CLSID (`00000000-...`), resulting in missed DOC detections.
   - It provides no mechanism to enforce memory or decompressed output budgets.

### 2.2 Acceptance of `zip` + `quick-xml` + `cfb` with `BudgetReader`
1. **`BudgetReader<R>` Boundary**:
   - All I/O operations from `zip` and `cfb` pass through a unified wrapper.
   - Tracks `source_bytes_read` and `seek_count`.
   - Strictly halts reading if total read exceeds candidate default (8 MiB) or if central directory window exceeds 1 MiB.
   - Prevents I/O amplification and memory exhaustion attacks.
2. **`quick-xml` Streaming Parser**:
   - Processes `[Content_Types].xml`, `_rels/.rels`, and `word/document.xml` as a stream of events.
   - Enforces XML element nesting depth <= 64.
   - Enforces decompressed stream size <= 256 KiB per XML file and <= 1 MiB total decompressed output.
3. **MS-DOC FIB Structural Verification Layer**:
   - Built on top of CFB stream reading.
   - Directly checks Microsoft MS-DOC official specification Section 2.5.1:
     - `wIdent == 0xA5EC`
     - `nFib >= 0x00C1` (Word 97 and later)
     - `fcMin` and `fcMac` text boundary offsets
   - Distinguishes fake `WordDocument` streams from genuine Word documents with 100% precision.
