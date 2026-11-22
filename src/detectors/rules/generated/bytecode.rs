use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static BYTECODE_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None,
        recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n035\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None,
        recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n037\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None,
        recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n038\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dex", category: "executable", container_id: None,
        recommended_ext: "dex", accepted_exts: &["dex"],
        patterns: &[MagicPattern::new(0, b"dex\n039\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.dex.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "luac", category: "executable", container_id: None,
        recommended_ext: "luac", accepted_exts: &["luac", "lua"],
        patterns: &[MagicPattern::new(0, b"\x1bLua")],
        min_header_len: 4, priority: 50, evidence_code: "rule.luac.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ar", category: "archive", container_id: None,
        recommended_ext: "a", accepted_exts: &["a", "lib", "ar"],
        patterns: &[MagicPattern::new(0, b"!<arch>\x0a")],
        min_header_len: 8, priority: 40, evidence_code: "rule.ar.magic",
        verification: Verification::Signature, validator: None,
    },
];
