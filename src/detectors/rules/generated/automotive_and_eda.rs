use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static AUTOMOTIVE_AND_EDA_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "mdf", category: "database", container_id: None, recommended_ext: "mdf", accepted_exts: &["mdf", "mf4"],
        patterns: &[MagicPattern::new(0, b"MDF4    ")], min_header_len: 8, priority: 50,
        evidence_code: "rule.mdf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vcd", category: "scientific", container_id: None, recommended_ext: "vcd", accepted_exts: &["vcd"],
        patterns: &[MagicPattern::new(0, b"$timescale")], min_header_len: 10, priority: 50,
        evidence_code: "rule.vcd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fsdb", category: "scientific", container_id: None, recommended_ext: "fsdb", accepted_exts: &["fsdb"],
        patterns: &[MagicPattern::new(0, b"FSDB")], min_header_len: 4, priority: 50,
        evidence_code: "rule.fsdb.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "edif", category: "scientific", container_id: None, recommended_ext: "edif", accepted_exts: &["edif"],
        patterns: &[MagicPattern::new(0, b"(edif ")], min_header_len: 6, priority: 50,
        evidence_code: "rule.edif.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "a2l", category: "scientific", container_id: None, recommended_ext: "a2l", accepted_exts: &["a2l"],
        patterns: &[MagicPattern::new(0, b"ASAP2_VERSION")], min_header_len: 13, priority: 50,
        evidence_code: "rule.a2l.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "svf", category: "hardware", container_id: None, recommended_ext: "svf", accepted_exts: &["svf"],
        patterns: &[MagicPattern::new(0, b"FREQUENCY ")], min_header_len: 10, priority: 50,
        evidence_code: "rule.svf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bit", category: "hardware", container_id: None, recommended_ext: "bit", accepted_exts: &["bit"],
        patterns: &[MagicPattern::new(0, b"\x00\x09\x0f\xf0\x0f\xf0\x0f\xf0\x0f\xf0\x00\x00\x01")], min_header_len: 13, priority: 50,
        evidence_code: "rule.bit.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dbc", category: "scientific", container_id: None, recommended_ext: "dbc", accepted_exts: &["dbc"],
        patterns: &[MagicPattern::new(0, b"VERSION \"\"")], min_header_len: 10, priority: 50,
        evidence_code: "rule.dbc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ldf", category: "scientific", container_id: None, recommended_ext: "ldf", accepted_exts: &["ldf"],
        patterns: &[MagicPattern::new(0, b"LIN_description_file;")], min_header_len: 21, priority: 50,
        evidence_code: "rule.ldf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bsdl", category: "hardware", container_id: None, recommended_ext: "bsd", accepted_exts: &["bsd", "bsdl"],
        patterns: &[MagicPattern::new(0, b"entity ")], min_header_len: 7, priority: 50,
        evidence_code: "rule.bsdl.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "jed", category: "hardware", container_id: None, recommended_ext: "jed", accepted_exts: &["jed"],
        patterns: &[MagicPattern::new(0, b"\x02\r\nNOTE JEDEC")], min_header_len: 12, priority: 50,
        evidence_code: "rule.jed.magic", verification: Verification::Signature, validator: None,
    },
];
