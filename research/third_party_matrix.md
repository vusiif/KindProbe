# Third-Party Evaluation Matrix (KindProbe P0-R)

**Author:** kindprobe_agy_developer  
**Date:** 2026-10-31  
**Scope:** §10.1, §10.8 Third-party candidate libraries and tools matrix.

---

| Project Name | Repository / URL | Language | Version Tested | License | `Read+Seek` Controllability | Allocation / Budget Risks | Sample Misidentifications (on 23 fixtures) | Reusable Modules / Logic | Decision |
|---|---|---|---|---|---|---|---|---|---|
| `file-format` | `https://github.com/mmalecot/file-format` | Rust | 0.29.0 | MIT / Apache-2.0 | Partial (implements `from_reader`, but internal readers do unconstrained reads) | Unbounded zip entry iteration; no memory caps on decompressed data | 2 false confirmations (`fake_docx_zip.zip`, `partial_ooxml_missing_rels.zip`); 2 missed | Reference list of MIME types and extensions only | **replace** / **reference_only** |
| `infer` | `https://github.com/bojand/infer` | Rust | 0.22.0 | MIT | In-memory slice only (`&[u8]`) | Requires reading full probe window into memory; no reader abstraction | 4 false confirmations (truncated PNG, ZIP, CFB, fake DOCX); 4 misidentifications (all CFB as `msi`) | Pure byte comparison helpers | **replace** / **reference_only** |
| `zip` | `https://github.com/zip-rs/zip2` | Rust | 2.6.1 | MIT | Full (`Read + Seek` required) | EOCD search reads backwards; can be bounded by wrapper | 0 (pure archive parser, doesn't claim format ID on its own) | Central directory parsing and stream decompression | **accept** |
| `quick-xml` | `https://github.com/tafia/quick-xml` | Rust | 0.36.2 | MIT | Full (`BufRead` streaming) | None with depth counter; does not build DOM | 0 (pure XML parser) | Event-driven streaming XML parser | **accept** |
| `cfb` | `https://github.com/mdsteele/rust-cfb` | Rust | 0.10.0 | MIT | Full (`Read + Seek` required) | Sector chains and directory trees could allocate; controllable via `BudgetReader` | 0 (pure container parser) | Sector allocation and compound document stream navigation | **accept** |
| `serde` & `serde_json` | `https://github.com/serde-rs/json` | Rust | `serde` 1.0.229, `serde_json` 1.0.154 | MIT / Apache-2.0 | N/A (data modeling) | Minimal; fixed structs prevent unbounded deserialization | 0 | Serialization for JSON report format | **accept** |
| `file` (libmagic) | `https://github.com/file/file` | C | 5.46 | BSD-2-Clause | File descriptor / buffer | None inside external process | 3 false confirmations (`fake_docx_zip.zip`, `partial_ooxml_missing_rels.zip`, `fake_doc_cfb.doc`) | Magic rules syntax reference | **reference_only** |
| `Siegfried` | `https://github.com/richardlehane/siegfried` | Go | 1.11.9 | Apache-2.0 | File path / stream | Can scan full files if unanchored | 3 false confirmations (`partial_ooxml_missing_rels.zip`, `partial_ooxml_missing_main_part.zip`, `fake_doc_cfb.doc`) | PRONOM signature baseline comparison | **reference_only** |
| `DROID` / PRONOM | `https://github.com/digital-preservation/droid` | Java / XML | V125 | Open Government Licence 2.0 (OGL-v2.0) | File path | Container XML requires unpacking | Misses relationship checks in container signature 1030 | Format PUIDs, byte sequences | **reference_only** |
