use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static NETWORK_MANAGEMENT_AND_SNMP_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "sflow_v5_dump", category: "network", container_id: None, recommended_ext: "sflow", accepted_exts: &["sflow"],
        patterns: &[MagicPattern::new(0, b"\x00\x00\x00\x05\x00\x00\x00\x01")], min_header_len: 8, priority: 50,
        evidence_code: "rule.sflow_v5_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "snmp_mib_definition", category: "network", container_id: None, recommended_ext: "mib", accepted_exts: &["mib"],
        patterns: &[MagicPattern::new(0, b"-- MIB DEFINITIONS\n")], min_header_len: 19, priority: 50,
        evidence_code: "rule.snmp_mib_definition.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "radius_dictionary", category: "network", container_id: None, recommended_ext: "raddict", accepted_exts: &["raddict"],
        patterns: &[MagicPattern::new(0, b"# RADIUS dictionary\n")], min_header_len: 20, priority: 50,
        evidence_code: "rule.radius_dictionary.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cisco_running_config", category: "network", container_id: None, recommended_ext: "cfg_cisco", accepted_exts: &["cfg_cisco"],
        patterns: &[MagicPattern::new(0, b"!\n! Last configuration change at")], min_header_len: 31, priority: 50,
        evidence_code: "rule.cisco_running_config.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "juniper_set_config", category: "network", container_id: None, recommended_ext: "cfg_juniper", accepted_exts: &["cfg_juniper"],
        patterns: &[MagicPattern::new(0, b"## Last commit: ")], min_header_len: 16, priority: 50,
        evidence_code: "rule.juniper_set_config.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wireshark_k12_text", category: "network", container_id: None, recommended_ext: "k12", accepted_exts: &["k12"],
        patterns: &[MagicPattern::new(0, b"+---------+---------------+---------------+\n")], min_header_len: 44, priority: 50,
        evidence_code: "rule.wireshark_k12_text.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "visual_networks_traffic", category: "network", container_id: None, recommended_ext: "vnlog", accepted_exts: &["vnlog"],
        patterns: &[MagicPattern::new(0, b"Visual Networks Traffic Capture\n")], min_header_len: 32, priority: 50,
        evidence_code: "rule.visual_networks_traffic.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "coad_network_log", category: "network", container_id: None, recommended_ext: "coad", accepted_exts: &["coad"],
        patterns: &[MagicPattern::new(0, b"COAD_LOG_V1")], min_header_len: 11, priority: 50,
        evidence_code: "rule.coad_network_log.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "argus_audit_record", category: "network", container_id: None, recommended_ext: "argus", accepted_exts: &["argus"],
        patterns: &[MagicPattern::new(0, b"ArgusRecord\x00")], min_header_len: 12, priority: 50,
        evidence_code: "rule.argus_audit_record.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ipfix_stream", category: "network", container_id: None, recommended_ext: "ipfix", accepted_exts: &["ipfix"],
        patterns: &[MagicPattern::new(0, b"IPFIX_STREAM_V1\x00")], min_header_len: 16, priority: 50,
        evidence_code: "rule.ipfix_stream.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "etherpeek_v7", category: "network", container_id: None, recommended_ext: "pkt_peek", accepted_exts: &["pkt_peek"],
        patterns: &[MagicPattern::new(0, b"\x7fPEEK\x07\x00")], min_header_len: 7, priority: 50,
        evidence_code: "rule.etherpeek_v7.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "airodump_ivs", category: "network", container_id: None, recommended_ext: "ivs", accepted_exts: &["ivs"],
        patterns: &[MagicPattern::new(0, b"\xbf\xca\x84\xd4")], min_header_len: 4, priority: 50,
        evidence_code: "rule.airodump_ivs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "netmon_2_capture", category: "network", container_id: None, recommended_ext: "netmon2", accepted_exts: &["netmon2"],
        patterns: &[MagicPattern::new(0, b"GMBU\x00\x00\x00\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.netmon_2_capture.magic", verification: Verification::Signature, validator: None,
    },
];
