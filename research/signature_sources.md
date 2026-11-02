# Signature Sources and Rule Review (KindProbe P0-R)

**Author:** kindprobe_agy_developer  
**Date:** 2026-10-31  
**Scope:** §10.4, §10.5, §10.8 Evaluation of signature databases, datasets, and licensing.

---

## 1. Primary Signature Sources Evaluated

| Dataset / Source | Canonical Link / Reference | Rule Format | Dataset Version | Download Artifact Hash (SHA256) | License / Attribution | Field Types Used | Supported Formats Evaluated | Imported Rules | Rejected / Deferred Rules & Reason |
|---|---|---|---|---|---|---|---|---|---|
| **PRONOM Binary Signatures** (The National Archives UK) | [PRONOM Signature List](https://pronom.nationalarchives.gov.uk/signature-list) | XML (`DROID_SignatureFile_V125.xml`) | V125 | Extracted from `data_1-11-9.zip` (`1a284a57d102996ab99e393dc40e7e213039ab44e1573c55fa9d9935840515cf`) | Open Government Licence v2.0 (OGL-v2.0) | `BOFoffset`, `Sequence`, `ByteSequence` | `fmt/11` (PNG), `fmt/18` (PDF 1.4), `x-fmt/263` (ZIP) | 3 rules (PNG magic, PDF magic, ZIP magic) | >1000 rules deferred: out of P0 scope (audio/video/executables); unanchored search rules rejected due to unbounded I/O. |
| **PRONOM Container Signatures** (The National Archives UK) | [PRONOM Container Signature](https://pronom.nationalarchives.gov.uk/signature-list) | XML (`container-signature-20260119.xml`) | 2026-01-19 | Extracted from `data_1-11-9.zip` (`1a284a57d102996ab99e393dc40e7e213039ab44e1573c55fa9d9935840515cf`) | Open Government Licence v2.0 (OGL-v2.0) | `ContainerType`, `File`, `InternalSignature` | Container 1030 (Word OOXML), Container 1020 (Word 97 OLE2) | 0 directly imported | **Rejected for direct import**: Container 1030 only checks `[Content_Types].xml` content type string, failing on missing `_rels/.rels` and missing main part. Container 1020 only checks `WordDocument` stream name, failing on fake streams without FIB. Replaced by KindProbe §4/§5 structured verification. |
| **Microsoft Official Specifications** (Microsoft Corporation) | [MS-DOC: Word (.doc) Binary File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-doc/acb08ace-f268-4879-a998-de592103a70e) | Technical Specification PDF/HTML | v20240521 (Section 2.5.1 FibBase) | Verified via Microsoft Open Specifications | Microsoft Open Specifications Promise / Copyright Microsoft Corp. (Technical documentation) | Binary structure fields (`wIdent`, `nFib`, `flags`, `fcMin`, `fcMac`) | Microsoft Word 97-2003 Binary Document (`doc`) | 1 structured rule (`sig.cfb.doc.fib97`) | 0 (fully adopted for structural FIB verification) |
| **ECMA-376 / ISO/IEC 29500** (ECMA International / ISO) | [ECMA-376 5th Edition](https://www.ecma-international.org/publications-and-standards/standards/ecma-376/) | International Standard Specification | Part 2 (Open Packaging Conventions) | Verified via ECMA International | ECMA Open Standards | XML Elements (`[Content_Types].xml`, `_rels/.rels`, `w:document`) | Microsoft Office Open XML Document (`docx`) | 1 structured rule (`sig.ooxml.docx.opc`) | 0 (fully adopted for OPC relationship & namespace verification) |
| **freedesktop shared-mime-info** | [freedesktop.org shared-mime-info](https://specifications.freedesktop.org/shared-mime-info/latest/) | XML (`freedesktop.org.xml`) | 2.4 | Extracted from `data_1-11-9.zip` | GNU GPL v2.0 or later | `<magic>`, `<match>` | PNG, PDF, ZIP | 0 imported | **Deferred**: GPL-2.0+ rule data cannot be embedded without licensing audit; glob rules forbidden by KindProbe §6.3. |

---

## 2. Rule Acceptance and Quarantine Policy

Following specification §10.5:
1. **Quarantine Unanchored Signatures**: Any rule with unbounded relative search offsets or non-zero min-offsets is quarantined to prevent full-file scanning.
2. **Quarantine Name-Only Container Matches**: PRONOM's Container 1020 and 1030 were proven in our benchmark to cause false positive DOC/DOCX detections. They are quarantined and replaced by KindProbe's multi-step structural verification.
3. **Clean Provenance**: In P0, only 5 verified rules are admitted into the engine registry (PNG, PDF, ZIP, DOCX OPC, DOC FIB). Each rule is mapped to formal specification documents and verified against 23 positive and negative fixtures.
