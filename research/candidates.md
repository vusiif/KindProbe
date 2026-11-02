# Candidate Analysis and Empirical Comparison (KindProbe P0-R)

**Author:** kindprobe_agy_developer  
**Date:** 2026-10-31  
**Scope:** Specification §10.3 / §11.2 P0-G0. Empirical evaluation of standalone tools (`file 5.46`, `Siegfried 1.11.9`) and Rust libraries (`infer 0.22.0`, `file-format 0.29.0`, `zip 2.6.1 + quick-xml 0.36.2`, `cfb 0.10.0`) on a fixed 23-fixture benchmark manifest.

---

## 1. Candidate Overview

| Candidate | Category | Version | Upstream / Source | Evaluated Role |
|---|---|---|---|---|
| `file` (libmagic) | CLI / C | 5.46 | Host system Debian package | External standalone baseline |
| `Siegfried` (`sf`) | CLI / Go | 1.11.9 | [GitHub richardlehane/siegfried](https://github.com/richardlehane/siegfried) (Release v1.11.9, PRONOM V125 & container-signature-20260119) | External standalone PRONOM baseline |
| `infer` | Rust Library | 0.22.0 | [crates.io/crates/infer](https://crates.io/crates/infer) | Lightweight magic signature baseline |
| `file-format` | Rust Library | 0.29.0 | [crates.io/crates/file-format](https://crates.io/crates/file-format) | Multi-format reader baseline |
| `zip` + `quick-xml` | Rust Library | `zip` 2.6.1, `quick-xml` 0.36.2 | [crates.io](https://crates.io) | Candidate OOXML / container parser layer |
| `cfb` | Rust Library | 0.10.0 | [crates.io/crates/cfb](https://crates.io/crates/cfb) | Candidate CFB container parser layer |

---

## 2. Empirical Benchmark Matrix (Summary of `baseline-results.json`)

Tested on 23 fixed synthetic fixtures generated via `scripts/generate_fixtures.py`.

### 2.1 DOCX vs ZIP Discrimination

| Fixture | True Expected | `file 5.46` | `Siegfried 1.11.9` | `infer 0.22.0` | `file-format 0.29.0` |
|---|---|---|---|---|---|
| `plain_zip.zip` | `zip` (format) | `Zip archive data` | `ZIP Format` | `zip` | `zip` |
| `fake_docx_zip.zip` (only `word/document.xml`) | `zip` (format) | **`Microsoft Word 2007+` (FALSE POSITIVE)** | `ZIP Format` | **`docx` (FALSE POSITIVE)** | **`docx` (FALSE POSITIVE)** |
| `partial_ooxml_missing_rels.zip` | `zip` (container_only) | **`Microsoft Word 2007+` (FALSE POSITIVE)** | **`Microsoft Word for Windows` (FALSE POSITIVE)** | `zip` (missed container diagnosis) | **`docx` (FALSE POSITIVE)** |
| `partial_ooxml_missing_main_part.zip` | `zip` (container_only) | `Zip archive data` | **`Microsoft Word for Windows` (FALSE POSITIVE)** | `zip` | `zip` |
| `true_docx.docx` | `docx` (format) | `Microsoft Word 2007+` | `Microsoft Word for Windows` | `zip` (failed to verify) | `docx` |
| `true_docx_strict.docx` | `docx` (format) | `Microsoft Word 2007+` | `DOCX Strict OOXML Document` | `zip` (failed to verify) | `docx` |
| `true_docx_rel_path.docx` | `docx` (format) | `Microsoft OOXML` | `Microsoft Word for Windows` | `zip` (failed to verify) | `zip` (missed relative path) |

**Key Findings:**
1. `file 5.46`, `infer 0.22.0`, and `file-format 0.29.0` all falsely identify `fake_docx_zip.zip` as DOCX because they merely scan for the string `"word/"` without checking `[Content_Types].xml` or package relationships.
2. `Siegfried` (PRONOM V125 container signature 1030) falsely identifies `partial_ooxml_missing_rels.zip` and `partial_ooxml_missing_main_part.zip` as Word because PRONOM container rules only scan for the byte sequence `ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"` inside `[Content_Types].xml`, ignoring missing `_rels/.rels` and missing main document parts!
3. `infer 0.22.0` fails to identify valid DOCX files when they don't follow its hardcoded local header offset pattern, returning only `zip`.
4. `file-format 0.29.0` fails on relative targets in relationships (e.g. `document.xml` at root).

### 2.2 DOC vs CFB Discrimination

| Fixture | True Expected | `file 5.46` | `Siegfried 1.11.9` | `infer 0.22.0` | `file-format 0.29.0` |
|---|---|---|---|---|---|
| `plain_cfb.cfb` | `cfb` (container_only) | `Composite Document File V2` | `OLE2 Compound Document Format` | **`msi` (MISIDENTIFIED)** | `cfb` |
| `fake_doc_cfb.doc` (stream name `WordDocument`, fake data) | `cfb` (container_only) | **`CDFV2 Microsoft Word` (FALSE POSITIVE)** | **`Microsoft Word (Generic)` (FALSE POSITIVE)** | **`msi` (MISIDENTIFIED)** | `cfb` |
| `true_doc.doc` (valid FIB wIdent `0xA5EC`, nFib `0x00C1`) | `doc` (format) | `CDFV2 Microsoft Word` | `Microsoft Word (Generic)` | **`msi` (MISIDENTIFIED)** | `cfb` (missed DOC) |
| `cfb_cyclic_chain.doc` (cyclic FAT chain) | `cfb` (container_only/error) | `CDFV2 Microsoft Word` | `Microsoft Word (Generic)` | **`msi` (MISIDENTIFIED)** | `cfb` |

**Key Findings:**
1. Both `file 5.46` and `Siegfried 1.11.9` rely entirely on the presence of the stream directory name `WordDocument`. Neither checks the File Information Block (FIB). Thus, a dummy CFB file with a stream named `WordDocument` containing garbage is falsely confirmed as Microsoft Word.
2. `infer 0.22.0` misidentifies all CFB compound files as `msi` because its CLSID matcher and OLE2 parser fail to inspect stream names or FIB.
3. `file-format 0.29.0` only inspects root entry CLSID (`00020900-...` or `00020906-...`). In standard Word 97 documents, the root entry CLSID is frequently all zeros (`00000000-0000-0000-0000-000000000000`), so `file-format` misses genuine DOC files and reports only generic `cfb`.

### 2.3 Truncation and Weak Signatures

| Fixture | True Expected | `file 5.46` | `Siegfried 1.11.9` | `infer 0.22.0` | `file-format 0.29.0` |
|---|---|---|---|---|---|
| `truncated_png.png` (4 bytes `\x89PNG`) | `unknown` / `none` | `Non-ISO extended-ASCII` | `empty` | **`png` (FALSE POSITIVE)** | `bin` |
| `truncated_zip.zip` (8 bytes) | `unknown` / `none` | `data` | `empty` | **`zip` (FALSE POSITIVE)** | `bin` |
| `truncated_cfb.doc` (12 bytes) | `unknown` / `none` | `OLE 2 Compound Document` | `empty` | **`msi` (FALSE POSITIVE)** | `bin` |

**Key Findings:**
`infer 0.22.0` matches short prefixes (e.g. 4 bytes for PNG, 8 bytes for ZIP) and immediately declares format confirmation without checking headers or structural markers.

---

## 3. Structural Defect Summary in Third-Party Tools

1. **`file 5.46`**: False confirmation count = 3 on our fixed benchmark.
   - Falsely confirms `fake_docx_zip.zip` as `Microsoft Word 2007+`.
   - Falsely confirms `partial_ooxml_missing_rels.zip` as `Microsoft Word 2007+`.
   - Falsely confirms `fake_doc_cfb.doc` as `CDFV2 Microsoft Word`.
2. **`Siegfried 1.11.9`**: False confirmation count = 3 on our fixed benchmark.
   - Falsely confirms `partial_ooxml_missing_rels.zip` as `Microsoft Word for Windows`.
   - Falsely confirms `partial_ooxml_missing_main_part.zip` as `Microsoft Word for Windows`.
   - Falsely confirms `fake_doc_cfb.doc` as `Microsoft Word (Generic)`.
3. **`infer 0.22.0`**: False confirmation count = 4, Misidentification count = 4.
   - Falsely confirms `fake_docx_zip.zip` as `docx`.
   - Falsely confirms `truncated_png.png` as `png`.
   - Falsely confirms `truncated_zip.zip` as `zip`.
   - Falsely confirms `truncated_cfb.doc` as `msi`.
   - Misidentifies genuine DOC, plain CFB, and fake DOC as `msi`.
4. **`file-format 0.29.0`**: False confirmation count = 2, Missed format count = 2.
   - Falsely confirms `fake_docx_zip.zip` as `docx`.
   - Falsely confirms `partial_ooxml_missing_rels.zip` as `docx`.
   - Misses `true_doc.doc` (returns generic `cfb`).
   - Misses `true_docx_rel_path.docx` (returns `zip`).

---

## 4. Architectural Conclusion for KindProbe

None of the existing tools or crates satisfy the KindProbe safety, budget, and accuracy contract:
- **Cannot directly embed `infer`**: Too loose, false confirmations on truncated inputs, no container-level analysis, no budget awareness.
- **Cannot directly embed `file-format`**: Scans zip file paths for `"word/"` and declares DOCX; ignores FIB in CFB; no resource budgeting.
- **Cannot rely on PRONOM/libmagic blindly**: Both PRONOM and libmagic rules have severe false-positive loopholes on partial OOXML and fake DOC streams.
- **KindProbe's Winning Strategy**:
  1. High-precision BOF signature matching for PNG and PDF (checking IHDR chunk for PNG and PDF version marker).
  2. Strict, bounded ZIP central directory parser wrapping `zip 2.6.1` and `quick-xml 0.36.2` with `BudgetReader` and decompression caps.
  3. Strict, bounded CFB parser with cycle detection, validating `WordDocument` stream FIB (`wIdent == 0xA5EC`, `nFib >= 0x00C1`).
  4. Separate container vs format identification, zero false confirmations on negative fixtures.
