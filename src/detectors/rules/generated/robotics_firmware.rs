use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static ROBOTICS_FIRMWARE_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "mcap", category: "scientific", container_id: None,
        recommended_ext: "mcap", accepted_exts: &["mcap"],
        patterns: &[MagicPattern::new(0, b"\x89MCAP0\x0d\x0a")],
        min_header_len: 8, priority: 50, evidence_code: "rule.mcap.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bag", category: "scientific", container_id: None,
        recommended_ext: "bag", accepted_exts: &["bag"],
        patterns: &[MagicPattern::new(0, b"#ROSBAG V2.0\x0a")],
        min_header_len: 13, priority: 50, evidence_code: "rule.bag.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pcd", category: "scientific", container_id: None,
        recommended_ext: "pcd", accepted_exts: &["pcd"],
        patterns: &[MagicPattern::new(0, b"# .PCD v")],
        min_header_len: 8, priority: 50, evidence_code: "rule.pcd.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dtb", category: "system", container_id: None,
        recommended_ext: "dtb", accepted_exts: &["dtb", "dtbo"],
        patterns: &[MagicPattern::new(0, b"\xd0\x0d\xfe\xed")],
        min_header_len: 4, priority: 50, evidence_code: "rule.dtb.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "uimage", category: "system", container_id: None,
        recommended_ext: "uimage", accepted_exts: &["uimage", "img"],
        patterns: &[MagicPattern::new(0, b"\x27\x05\x19\x56")],
        min_header_len: 4, priority: 50, evidence_code: "rule.uimage.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cram", category: "scientific", container_id: None,
        recommended_ext: "cram", accepted_exts: &["cram"],
        patterns: &[MagicPattern::new(0, b"CRAM")],
        min_header_len: 4, priority: 50, evidence_code: "rule.cram.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bcf", category: "scientific", container_id: None,
        recommended_ext: "bcf", accepted_exts: &["bcf"],
        patterns: &[MagicPattern::new(0, b"BCF\x02")],
        min_header_len: 4, priority: 50, evidence_code: "rule.bcf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "blf", category: "system", container_id: None,
        recommended_ext: "blf", accepted_exts: &["blf"],
        patterns: &[MagicPattern::new(0, b"LOGG")],
        min_header_len: 4, priority: 50, evidence_code: "rule.blf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gbr", category: "model", container_id: None,
        recommended_ext: "gbr", accepted_exts: &["gbr", "ger"],
        patterns: &[MagicPattern::new(0, b"%FS")],
        min_header_len: 3, priority: 50, evidence_code: "rule.gbr.magic",
        verification: Verification::Signature, validator: None,
    },
];
