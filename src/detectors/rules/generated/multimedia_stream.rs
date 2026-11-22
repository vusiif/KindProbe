use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static MULTIMEDIA_STREAM_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "asf", category: "video", container_id: None,
        recommended_ext: "wmv", accepted_exts: &["wmv", "wma", "asf"],
        patterns: &[MagicPattern::new(0, b"\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c")],
        min_header_len: 16, priority: 50, evidence_code: "rule.asf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "rm", category: "video", container_id: None,
        recommended_ext: "rm", accepted_exts: &["rm", "rmvb", "ra"],
        patterns: &[MagicPattern::new(0, b".RMF")],
        min_header_len: 4, priority: 50, evidence_code: "rule.rm.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "swf", category: "video", container_id: None,
        recommended_ext: "swf", accepted_exts: &["swf"],
        patterns: &[MagicPattern::new(0, b"FWS")],
        min_header_len: 3, priority: 50, evidence_code: "rule.swf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "swf", category: "video", container_id: None,
        recommended_ext: "swf", accepted_exts: &["swf"],
        patterns: &[MagicPattern::new(0, b"CWS")],
        min_header_len: 3, priority: 50, evidence_code: "rule.swf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "swf", category: "video", container_id: None,
        recommended_ext: "swf", accepted_exts: &["swf"],
        patterns: &[MagicPattern::new(0, b"ZWS")],
        min_header_len: 3, priority: 50, evidence_code: "rule.swf.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dts", category: "audio", container_id: None,
        recommended_ext: "dts", accepted_exts: &["dts", "cda"],
        patterns: &[MagicPattern::new(0, b"\x7f\xfe\x80\x01")],
        min_header_len: 4, priority: 50, evidence_code: "rule.dts.magic",
        verification: Verification::Signature, validator: None,
    },
];
