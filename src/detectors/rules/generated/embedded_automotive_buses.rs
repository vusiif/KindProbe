use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static EMBEDDED_AUTOMOTIVE_BUSES_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "can_candbc_v2", category: "automotive", container_id: None, recommended_ext: "candbc", accepted_exts: &["candbc"],
        patterns: &[MagicPattern::new(0, b"CANdb++ V2.0\n")], min_header_len: 13, priority: 50,
        evidence_code: "rule.can_candbc_v2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "lin_protocol_cfg", category: "automotive", container_id: None, recommended_ext: "lincfg", accepted_exts: &["lincfg"],
        patterns: &[MagicPattern::new(0, b"LIN_CLUSTER_CONFIG_V1\n")], min_header_len: 22, priority: 50,
        evidence_code: "rule.lin_protocol_cfg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fibex_xml", category: "automotive", container_id: None, recommended_ext: "fibex", accepted_exts: &["fibex"],
        patterns: &[MagicPattern::new(0, b"<FIBEX xmlns=\"http://www.asam.net/xml/fbx\"")], min_header_len: 42, priority: 50,
        evidence_code: "rule.fibex_xml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "autosar_arxml", category: "automotive", container_id: None, recommended_ext: "arxml", accepted_exts: &["arxml"],
        patterns: &[MagicPattern::new(0, b"<AUTOSAR xmlns=\"http://autosar.org/schema/r4.0\"")], min_header_len: 48, priority: 50,
        evidence_code: "rule.autosar_arxml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vector_asc_log", category: "automotive", container_id: None, recommended_ext: "asc_can", accepted_exts: &["asc_can"],
        patterns: &[MagicPattern::new(0, b"// Vector Informatik CAN Log\n")], min_header_len: 29, priority: 50,
        evidence_code: "rule.vector_asc_log.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "kvaser_memorator_kme", category: "automotive", container_id: None, recommended_ext: "kme", accepted_exts: &["kme"],
        patterns: &[MagicPattern::new(0, b"KVASER_MEMORATOR_LOG_V1\x00")], min_header_len: 24, priority: 50,
        evidence_code: "rule.kvaser_memorator_kme.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "peak_can_trc", category: "automotive", container_id: None, recommended_ext: "trc_pcan", accepted_exts: &["trc_pcan"],
        patterns: &[MagicPattern::new(0, b";$FILEVERSION=1.1\n;$STARTTIME=")], min_header_len: 30, priority: 50,
        evidence_code: "rule.peak_can_trc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "can_open_eds", category: "automotive", container_id: None, recommended_ext: "eds_co", accepted_exts: &["eds_co"],
        patterns: &[MagicPattern::new(0, b"[FileInfo]\nCreatedBy=CANopen")], min_header_len: 26, priority: 50,
        evidence_code: "rule.can_open_eds.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "can_open_dcf", category: "automotive", container_id: None, recommended_ext: "dcf_co", accepted_exts: &["dcf_co"],
        patterns: &[MagicPattern::new(0, b"[FileInfo]\nFileName=CANopen.dcf")], min_header_len: 30, priority: 50,
        evidence_code: "rule.can_open_dcf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "canmatrix_sym", category: "automotive", container_id: None, recommended_ext: "sym_can", accepted_exts: &["sym_can"],
        patterns: &[MagicPattern::new(0, b"FormatVersion=5.0\n// Symbols\n")], min_header_len: 28, priority: 50,
        evidence_code: "rule.canmatrix_sym.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "candump_log", category: "automotive", container_id: None, recommended_ext: "candump", accepted_exts: &["candump"],
        patterns: &[MagicPattern::new(0, b"(000000.000000) can0 ")], min_header_len: 22, priority: 50,
        evidence_code: "rule.candump_log.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "asam_mdf2", category: "automotive", container_id: None, recommended_ext: "mdf2", accepted_exts: &["mdf2"],
        patterns: &[MagicPattern::new(0, b"MDF     2.00    ")], min_header_len: 16, priority: 50,
        evidence_code: "rule.asam_mdf2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "asam_mdf3", category: "automotive", container_id: None, recommended_ext: "mdf3", accepted_exts: &["mdf3"],
        patterns: &[MagicPattern::new(0, b"MDF     3.30    ")], min_header_len: 16, priority: 50,
        evidence_code: "rule.asam_mdf3.magic", verification: Verification::Signature, validator: None,
    },
];
