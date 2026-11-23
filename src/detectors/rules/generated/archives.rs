use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static ARCHIVES_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "cpio", category: "archive", container_id: None,
        recommended_ext: "cpio", accepted_exts: &["cpio"],
        patterns: &[MagicPattern::new(0, b"070707")],
        min_header_len: 6, priority: 50, evidence_code: "rule.cpio.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "lz", category: "archive", container_id: None,
        recommended_ext: "lz", accepted_exts: &["lz"],
        patterns: &[MagicPattern::new(0, b"LZIP")],
        min_header_len: 4, priority: 50, evidence_code: "rule.lz.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "arj", category: "archive", container_id: None,
        recommended_ext: "arj", accepted_exts: &["arj"],
        patterns: &[MagicPattern::new(0, b"`\xea")],
        min_header_len: 2, priority: 50, evidence_code: "rule.arj.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "zoo", category: "archive", container_id: None,
        recommended_ext: "zoo", accepted_exts: &["zoo"],
        patterns: &[MagicPattern::new(20, b"\xdc\xa7\xc4\xfd")],
        min_header_len: 24, priority: 50, evidence_code: "rule.zoo.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cab", category: "archive", container_id: None,
        recommended_ext: "cab", accepted_exts: &["cab"],
        patterns: &[MagicPattern::new(0, b"MSCF")],
        min_header_len: 4, priority: 50, evidence_code: "rule.cab.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wim", category: "archive", container_id: None,
        recommended_ext: "wim", accepted_exts: &["wim", "swm", "esd"],
        patterns: &[MagicPattern::new(0, b"MSWIM\x00\x00\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.wim.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "sit", category: "archive", container_id: None,
        recommended_ext: "sit", accepted_exts: &["sit"],
        patterns: &[MagicPattern::new(0, b"StuffIt")],
        min_header_len: 7, priority: 50, evidence_code: "rule.sit.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "sqx", category: "archive", container_id: None,
        recommended_ext: "sqx", accepted_exts: &["sqx"],
        patterns: &[MagicPattern::new(0, b"SQX\x00")],
        min_header_len: 4, priority: 50, evidence_code: "rule.sqx.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pack200", category: "archive", container_id: None, recommended_ext: "pack", accepted_exts: &["pack"],
        patterns: &[MagicPattern::new(0, b"\xca\xfe\xd0\x0d")], min_header_len: 4,
        priority: 50, evidence_code: "rule.pack200.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "isc", category: "archive", container_id: None, recommended_ext: "isc", accepted_exts: &["isc", "hdr"],
        patterns: &[MagicPattern::new(0, b"ISc(")], min_header_len: 4,
        priority: 50, evidence_code: "rule.isc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "rpa", category: "archive", container_id: None,
        recommended_ext: "rpa", accepted_exts: &["rpa"],
        patterns: &[MagicPattern::new(0, b"RPA-")],
        min_header_len: 4, priority: 50, evidence_code: "rule.rpa.magic",
        verification: Verification::Signature, validator: None,
    },
];
