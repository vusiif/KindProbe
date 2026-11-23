use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static FIRMWARE_AND_FORENSICS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "rkfw", category: "firmware", container_id: None, recommended_ext: "rkfw", accepted_exts: &["rkfw"],
        patterns: &[MagicPattern::new(0, b"RKFW")], min_header_len: 4, priority: 50,
        evidence_code: "rule.rkfw.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cbfs", category: "firmware", container_id: None, recommended_ext: "cbfs", accepted_exts: &["cbfs"],
        patterns: &[MagicPattern::new(0, b"ORBC")], min_header_len: 4, priority: 50,
        evidence_code: "rule.cbfs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "trx", category: "firmware", container_id: None, recommended_ext: "trx", accepted_exts: &["trx"],
        patterns: &[MagicPattern::new(0, b"HDR0")], min_header_len: 4, priority: 50,
        evidence_code: "rule.trx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ubnt", category: "firmware", container_id: None, recommended_ext: "ubnt", accepted_exts: &["ubnt"],
        patterns: &[MagicPattern::new(0, b"OPEN")], min_header_len: 4, priority: 50,
        evidence_code: "rule.ubnt.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "qnxifs", category: "firmware", container_id: None, recommended_ext: "ifs", accepted_exts: &["ifs"],
        patterns: &[MagicPattern::new(0, b"\xeb\x7e\xff\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.qnxifs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "simg", category: "disk", container_id: None, recommended_ext: "simg", accepted_exts: &["simg"],
        patterns: &[MagicPattern::new(0, b"\x3a\xff\x26\xed")], min_header_len: 4, priority: 50,
        evidence_code: "rule.simg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bootimg", category: "firmware", container_id: None, recommended_ext: "bootimg", accepted_exts: &["bootimg"],
        patterns: &[MagicPattern::new(0, b"ANDROID!")], min_header_len: 8, priority: 50,
        evidence_code: "rule.bootimg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "evtx", category: "security", container_id: None, recommended_ext: "evtx", accepted_exts: &["evtx"],
        patterns: &[MagicPattern::new(0, b"ElfFile\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.evtx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "hve", category: "system", container_id: None, recommended_ext: "hve", accepted_exts: &["hve"],
        patterns: &[MagicPattern::new(0, b"regf")], min_header_len: 4, priority: 50,
        evidence_code: "rule.hve.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "aff", category: "security", container_id: None, recommended_ext: "aff", accepted_exts: &["aff"],
        patterns: &[MagicPattern::new(0, b"AFF\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.aff.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ewf", category: "security", container_id: None, recommended_ext: "e01", accepted_exts: &["e01", "ewf"],
        patterns: &[MagicPattern::new(0, b"EVF\x09\x0d\x0a\xff\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.ewf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "yac", category: "security", container_id: None, recommended_ext: "yac", accepted_exts: &["yac"],
        patterns: &[MagicPattern::new(0, b"YARA")], min_header_len: 4, priority: 50,
        evidence_code: "rule.yac.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "xnb", category: "game", container_id: None, recommended_ext: "xnb", accepted_exts: &["xnb"],
        patterns: &[MagicPattern::new(0, b"XNBw")], min_header_len: 4, priority: 50,
        evidence_code: "rule.xnb.magic", verification: Verification::Signature, validator: None,
    },
];
