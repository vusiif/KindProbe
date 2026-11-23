use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static DOCUMENTS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "dicom", category: "document", container_id: None, recommended_ext: "dcm", accepted_exts: &["dcm", "dicm"],
        patterns: &[MagicPattern::new(128, b"DICM")],
        min_header_len: 132, priority: 50, evidence_code: "rule.dicom.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "postscript", category: "document", container_id: None, recommended_ext: "ps", accepted_exts: &["ps", "eps", "ai"],
        patterns: &[MagicPattern::new(0, b"%!PS-Adobe-")],
        min_header_len: 11, priority: 50, evidence_code: "rule.postscript.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "rtf", category: "document", container_id: None, recommended_ext: "rtf", accepted_exts: &["rtf"],
        patterns: &[MagicPattern::new(0, b"{\x5crtf")],
        min_header_len: 5, priority: 50, evidence_code: "rule.rtf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dwg", category: "document", container_id: None, recommended_ext: "dwg", accepted_exts: &["dwg"],
        patterns: &[MagicPattern::new(0, b"AC10")],
        min_header_len: 4, priority: 50, evidence_code: "rule.dwg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dxf_bin", category: "document", container_id: None,
        recommended_ext: "dxf", accepted_exts: &["dxf"],
        patterns: &[MagicPattern::new(0, b"AutoCAD Binary DXF")],
        min_header_len: 18, priority: 50, evidence_code: "rule.dxf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dwf", category: "document", container_id: None,
        recommended_ext: "dwf", accepted_exts: &["dwf"],
        patterns: &[MagicPattern::new(0, b"(DWF V")],
        min_header_len: 6, priority: 50, evidence_code: "rule.dwf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wri", category: "document", container_id: None,
        recommended_ext: "wri", accepted_exts: &["wri"],
        patterns: &[MagicPattern::new(0, b"1\xbe\x00\x00")],
        min_header_len: 4, priority: 50, evidence_code: "rule.wri.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "xar", category: "document", container_id: None,
        recommended_ext: "xar", accepted_exts: &["xar"],
        patterns: &[MagicPattern::new(0, b"xar!")],
        min_header_len: 4, priority: 50, evidence_code: "rule.xar.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "hlp", category: "document", container_id: None, recommended_ext: "hlp", accepted_exts: &["hlp"],
        patterns: &[MagicPattern::new(0, b"?_\x03\x00")], min_header_len: 4,
        priority: 50, evidence_code: "rule.hlp.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "lit", category: "ebook", container_id: None, recommended_ext: "lit", accepted_exts: &["lit"],
        patterns: &[MagicPattern::new(0, b"ITOLITLS")], min_header_len: 8,
        priority: 50, evidence_code: "rule.lit.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fb2", category: "document", container_id: None,
        recommended_ext: "fb2", accepted_exts: &["fb2"],
        patterns: &[MagicPattern::new(0, b"<FictionBook")],
        min_header_len: 12, priority: 50, evidence_code: "rule.fb2.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "warc", category: "document", container_id: None, recommended_ext: "warc",
        accepted_exts: &["warc"], patterns: &[MagicPattern::new(0, b"WARC/1.")],
        min_header_len: 7, priority: 50, evidence_code: "rule.warc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "winhelp", category: "document", container_id: None, recommended_ext: "hlp",
        accepted_exts: &["hlp"], patterns: &[MagicPattern::new(0, b"?_\x00\x00")],
        min_header_len: 4, priority: 50, evidence_code: "rule.hlp.magic", verification: Verification::Signature, validator: None,
    },
];
