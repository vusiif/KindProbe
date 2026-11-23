use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static INDUSTRIAL_PLC_AND_SCADA_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "s7p", category: "engineering", container_id: None, recommended_ext: "s7p", accepted_exts: &["s7p"],
        patterns: &[MagicPattern::new(0, b"S7P\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.s7p.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pro_codesys", category: "engineering", container_id: None, recommended_ext: "pro", accepted_exts: &["pro"],
        patterns: &[MagicPattern::new(0, b"CoDeSys")], min_header_len: 7, priority: 50,
        evidence_code: "rule.pro_codesys.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mer", category: "engineering", container_id: None, recommended_ext: "mer", accepted_exts: &["mer"],
        patterns: &[MagicPattern::new(0, b"\x4d\x45\x52\x00\x01\x00")], min_header_len: 6, priority: 50,
        evidence_code: "rule.mer.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "apa", category: "engineering", container_id: None, recommended_ext: "apa", accepted_exts: &["apa"],
        patterns: &[MagicPattern::new(0, b"APA_ARC")], min_header_len: 7, priority: 50,
        evidence_code: "rule.apa.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cxp", category: "engineering", container_id: None, recommended_ext: "cxp", accepted_exts: &["cxp"],
        patterns: &[MagicPattern::new(0, b"OMRON CX-Programmer")], min_header_len: 19, priority: 50,
        evidence_code: "rule.cxp.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cxd", category: "engineering", container_id: None, recommended_ext: "cxd", accepted_exts: &["cxd"],
        patterns: &[MagicPattern::new(0, b"OMRON CX-Designer")], min_header_len: 17, priority: 50,
        evidence_code: "rule.cxd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mwp", category: "engineering", container_id: None, recommended_ext: "mwp", accepted_exts: &["mwp"],
        patterns: &[MagicPattern::new(4, b"S7200")], min_header_len: 9, priority: 50,
        evidence_code: "rule.mwp.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "eds", category: "engineering", container_id: None, recommended_ext: "eds", accepted_exts: &["eds"],
        patterns: &[MagicPattern::new(0, b"[FileInfo]")], min_header_len: 10, priority: 50,
        evidence_code: "rule.eds.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gsd", category: "engineering", container_id: None, recommended_ext: "gsd", accepted_exts: &["gsd"],
        patterns: &[MagicPattern::new(0, b"#Profibus")], min_header_len: 9, priority: 50,
        evidence_code: "rule.gsd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "xcos", category: "engineering", container_id: None, recommended_ext: "xcos", accepted_exts: &["xcos"],
        patterns: &[MagicPattern::new(0, b"<XcosDiagram")], min_header_len: 12, priority: 50,
        evidence_code: "rule.xcos.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "s7opt", category: "engineering", container_id: None, recommended_ext: "s7opt", accepted_exts: &["s7opt"],
        patterns: &[MagicPattern::new(0, b"S7_OPTION_FILE")], min_header_len: 14, priority: 50,
        evidence_code: "rule.s7opt.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "smc", category: "engineering", container_id: None, recommended_ext: "smc", accepted_exts: &["smc"],
        patterns: &[MagicPattern::new(0, b"SCHNEIDER_SMC")], min_header_len: 13, priority: 50,
        evidence_code: "rule.smc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pcu", category: "engineering", container_id: None, recommended_ext: "pcu", accepted_exts: &["pcu"],
        patterns: &[MagicPattern::new(0, b"%_N_")], min_header_len: 4, priority: 50,
        evidence_code: "rule.pcu.magic", verification: Verification::Signature, validator: None,
    },
];
