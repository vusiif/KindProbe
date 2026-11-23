use crate::detectors::rules::types::{MagicPattern, SignatureRule, ValidationStatus};
use crate::model::Verification;

fn validate_pyc(header: &[u8]) -> ValidationStatus {
    if header.len() < 16 {
        return ValidationStatus::Truncated;
    }
    if header[2] == 0x0d && header[3] == 0x0a && (header[1] == 0x0d || header[1] == 0xf3) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub static BYTECODE_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None, recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n035\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None, recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n037\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None, recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n038\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None, recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n039\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "luac", category: "executable", container_id: None, recommended_ext: "luac", accepted_exts: &["luac", "lua"],
        patterns: &[MagicPattern::new(0, b"\x1bLua")],
        min_header_len: 4, priority: 50, evidence_code: "rule.luac.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ar", category: "archive", container_id: None,
        recommended_ext: "a", accepted_exts: &["a", "lib", "ar"],
        patterns: &[MagicPattern::new(0, b"!<arch>\x0a")],
        min_header_len: 8, priority: 40, evidence_code: "rule.ar.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "beam", category: "executable", container_id: None,
        recommended_ext: "beam", accepted_exts: &["beam"],
        patterns: &[MagicPattern::new(0, b"FOR1"), MagicPattern::new(8, b"BEAM")],
        min_header_len: 12, priority: 50, evidence_code: "rule.beam.magic",
        verification: Verification::Structure, validator: None,
    },
    SignatureRule {
        format_id: "pyc", category: "executable", container_id: None,
        recommended_ext: "pyc", accepted_exts: &["pyc", "pyo"],
        patterns: &[MagicPattern::new(2, b"\x0d\x0a")],
        min_header_len: 16, priority: 45, evidence_code: "rule.pyc.magic",
        verification: Verification::Structure, validator: Some(validate_pyc),
    },
    SignatureRule {
        format_id: "mo", category: "translation", container_id: None,
        recommended_ext: "mo", accepted_exts: &["mo", "gmo"],
        patterns: &[MagicPattern::new(0, b"\xde\x12\x04\x95")],
        min_header_len: 4, priority: 50, evidence_code: "rule.mo.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bc", category: "bytecode", container_id: None, recommended_ext: "bc",
        accepted_exts: &["bc"], patterns: &[MagicPattern::new(0, b"BC\xc0\xde")],
        min_header_len: 4, priority: 50, evidence_code: "rule.bc.llvm", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "jmod", category: "bytecode", container_id: None, recommended_ext: "jmod",
        accepted_exts: &["jmod"], patterns: &[MagicPattern::new(0, b"JM\x01\x00")],
        min_header_len: 4, priority: 50, evidence_code: "rule.jmod.magic", verification: Verification::Signature, validator: None,
    },
];
