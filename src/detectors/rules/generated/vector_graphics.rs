use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static VECTOR_GRAPHICS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "wmf", category: "image", container_id: None,
        recommended_ext: "wmf", accepted_exts: &["wmf"],
        patterns: &[MagicPattern::new(0, b"\xd7\xcd\xc6\x9a")],
        min_header_len: 4, priority: 50, evidence_code: "rule.wmf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "emf", category: "image", container_id: None,
        recommended_ext: "emf", accepted_exts: &["emf"],
        patterns: &[MagicPattern::new(0, b"\x01\x00\x00\x00"), MagicPattern::new(40, b" EMF")],
        min_header_len: 44, priority: 50, evidence_code: "rule.emf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "qoi", category: "image", container_id: None,
        recommended_ext: "qoi", accepted_exts: &["qoi"],
        patterns: &[MagicPattern::new(0, b"qoif")],
        min_header_len: 4, priority: 50, evidence_code: "rule.qoi.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "jxl", category: "image", container_id: None,
        recommended_ext: "jxl", accepted_exts: &["jxl"],
        patterns: &[MagicPattern::new(0, b"\xff\x0a")],
        min_header_len: 2, priority: 50, evidence_code: "rule.jxl.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "jxl", category: "image", container_id: None,
        recommended_ext: "jxl", accepted_exts: &["jxl"],
        patterns: &[MagicPattern::new(0, b"\x00\x00\x00\x0cJXL \r\n\x87\n")],
        min_header_len: 12, priority: 50, evidence_code: "rule.jxl.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "flif", category: "image", container_id: None,
        recommended_ext: "flif", accepted_exts: &["flif"],
        patterns: &[MagicPattern::new(0, b"FLIF")],
        min_header_len: 4, priority: 50, evidence_code: "rule.flif.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fig", category: "image", container_id: None,
        recommended_ext: "fig", accepted_exts: &["fig"],
        patterns: &[MagicPattern::new(0, b"#FIG ")],
        min_header_len: 5, priority: 50, evidence_code: "rule.fig.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "flc", category: "video", container_id: None,
        recommended_ext: "flc", accepted_exts: &["flc", "fli"],
        patterns: &[MagicPattern::new(4, b"\x12\xaf")],
        min_header_len: 6, priority: 50, evidence_code: "rule.flc.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fli", category: "video", container_id: None,
        recommended_ext: "fli", accepted_exts: &["fli", "flc"],
        patterns: &[MagicPattern::new(4, b"\x11\xaf")],
        min_header_len: 6, priority: 50, evidence_code: "rule.fli.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "eps_bin", category: "image", container_id: None,
        recommended_ext: "eps", accepted_exts: &["eps"],
        patterns: &[MagicPattern::new(0, b"\xc5\xd0\xd3\xc6")],
        min_header_len: 4, priority: 50, evidence_code: "rule.eps_bin.magic",
        verification: Verification::Signature, validator: None,
    },
];
