use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static TEXTURES_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "dds", category: "image", container_id: None,
        recommended_ext: "dds", accepted_exts: &["dds"],
        patterns: &[MagicPattern::new(0, b"DDS ")],
        min_header_len: 4, priority: 50, evidence_code: "rule.dds.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ktx", category: "image", container_id: None,
        recommended_ext: "ktx", accepted_exts: &["ktx"],
        patterns: &[MagicPattern::new(0, b"\xabKTX 11\xbb\r\n\x1a\n")],
        min_header_len: 12, priority: 50, evidence_code: "rule.ktx.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ktx2", category: "image", container_id: None,
        recommended_ext: "ktx2", accepted_exts: &["ktx2"],
        patterns: &[MagicPattern::new(0, b"\xabKTX 20\xbb\r\n\x1a\n")],
        min_header_len: 12, priority: 50, evidence_code: "rule.ktx2.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "astc", category: "image", container_id: None,
        recommended_ext: "astc", accepted_exts: &["astc"],
        patterns: &[MagicPattern::new(0, b"\x13\xab\xa1\x5c")],
        min_header_len: 4, priority: 50, evidence_code: "rule.astc.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pvr", category: "image", container_id: None,
        recommended_ext: "pvr", accepted_exts: &["pvr"],
        patterns: &[MagicPattern::new(0, b"PVR\x03")],
        min_header_len: 4, priority: 50, evidence_code: "rule.pvr.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vtf", category: "image", container_id: None,
        recommended_ext: "vtf", accepted_exts: &["vtf"],
        patterns: &[MagicPattern::new(0, b"VTF\x00")],
        min_header_len: 16, priority: 50, evidence_code: "rule.vtf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pkm", category: "image", container_id: None,
        recommended_ext: "pkm", accepted_exts: &["pkm"],
        patterns: &[MagicPattern::new(0, b"PKM ")],
        min_header_len: 6, priority: 50, evidence_code: "rule.pkm.magic",
        verification: Verification::Signature, validator: None,
    },
];
