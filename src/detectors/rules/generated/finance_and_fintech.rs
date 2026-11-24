use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static FINANCE_AND_FINTECH_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "fix_protocol", category: "finance", container_id: None, recommended_ext: "fix", accepted_exts: &["fix"],
        patterns: &[MagicPattern::new(0, b"8=FIX.4.")], min_header_len: 8, priority: 50,
        evidence_code: "rule.fix_protocol.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mt940_statement", category: "finance", container_id: None, recommended_ext: "mt940", accepted_exts: &["mt940"],
        patterns: &[MagicPattern::new(0, b":20:START\r\n:25:")], min_header_len: 15, priority: 50,
        evidence_code: "rule.mt940_statement.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "swift_mt_msg", category: "finance", container_id: None, recommended_ext: "swift", accepted_exts: &["swift"],
        patterns: &[MagicPattern::new(0, b"{1:F01")], min_header_len: 7, priority: 50,
        evidence_code: "rule.swift_mt_msg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fints_message", category: "finance", container_id: None, recommended_ext: "fints", accepted_exts: &["fints"],
        patterns: &[MagicPattern::new(0, b"HNHBK:1:3+")], min_header_len: 10, priority: 50,
        evidence_code: "rule.fints_message.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gnucash_xml", category: "finance", container_id: None, recommended_ext: "gnucash", accepted_exts: &["gnucash"],
        patterns: &[MagicPattern::new(0, b"<gnc-v2")], min_header_len: 7, priority: 50,
        evidence_code: "rule.gnucash_xml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "iso20022_pain001", category: "finance", container_id: None, recommended_ext: "pain001", accepted_exts: &["pain001"],
        patterns: &[MagicPattern::new(0, b"<Document xmlns=\"urn:iso:std:iso:20022:tech:xsd:pain.001")], min_header_len: 55, priority: 50,
        evidence_code: "rule.iso20022_pain001.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "iso20022_camt053", category: "finance", container_id: None, recommended_ext: "camt053", accepted_exts: &["camt053"],
        patterns: &[MagicPattern::new(0, b"<Document xmlns=\"urn:iso:std:iso:20022:tech:xsd:camt.053")], min_header_len: 55, priority: 50,
        evidence_code: "rule.iso20022_camt053.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fpml_financial", category: "finance", container_id: None, recommended_ext: "fpml", accepted_exts: &["fpml"],
        patterns: &[MagicPattern::new(0, b"<dataDocument fpmlVersion=")], min_header_len: 26, priority: 50,
        evidence_code: "rule.fpml_financial.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "finvoice_xml", category: "finance", container_id: None, recommended_ext: "finvoice", accepted_exts: &["finvoice"],
        patterns: &[MagicPattern::new(0, b"<Finvoice Version=")], min_header_len: 18, priority: 50,
        evidence_code: "rule.finvoice_xml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "facturx_invoice", category: "finance", container_id: None, recommended_ext: "facturx", accepted_exts: &["facturx"],
        patterns: &[MagicPattern::new(0, b"<rsm:CrossIndustryInvoice")], min_header_len: 25, priority: 50,
        evidence_code: "rule.facturx_invoice.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ebics_request", category: "finance", container_id: None, recommended_ext: "ebics", accepted_exts: &["ebics"],
        patterns: &[MagicPattern::new(0, b"<ebicsRequest xmlns=\"urn:org:ebics:H004\"")], min_header_len: 40, priority: 50,
        evidence_code: "rule.ebics_request.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bloomberg_data", category: "finance", container_id: None, recommended_ext: "bbg", accepted_exts: &["bbg"],
        patterns: &[MagicPattern::new(0, b"START-OF-FILE\r\n")], min_header_len: 15, priority: 50,
        evidence_code: "rule.bloomberg_data.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "reuters_data", category: "finance", container_id: None, recommended_ext: "ric", accepted_exts: &["ric"],
        patterns: &[MagicPattern::new(0, b"<RIC_DATA_FILE>")], min_header_len: 15, priority: 50,
        evidence_code: "rule.reuters_data.magic", verification: Verification::Signature, validator: None,
    },
];
