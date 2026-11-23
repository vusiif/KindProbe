use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static BIO_AND_CREATIVE_DESIGN_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "nii", category: "scientific", container_id: None, recommended_ext: "nii", accepted_exts: &["nii"],
        patterns: &[MagicPattern::new(344, b"n+1\x00")], min_header_len: 348, priority: 50,
        evidence_code: "rule.nii.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ab1", category: "scientific", container_id: None, recommended_ext: "ab1", accepted_exts: &["ab1"],
        patterns: &[MagicPattern::new(0, b"ABIF")], min_header_len: 4, priority: 50,
        evidence_code: "rule.ab1.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fcs", category: "scientific", container_id: None, recommended_ext: "fcs", accepted_exts: &["fcs"],
        patterns: &[MagicPattern::new(0, b"FCS3.0")], min_header_len: 6, priority: 50,
        evidence_code: "rule.fcs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cdx", category: "scientific", container_id: None, recommended_ext: "cdx", accepted_exts: &["cdx"],
        patterns: &[MagicPattern::new(0, b"VjCD0100")], min_header_len: 8, priority: 50,
        evidence_code: "rule.cdx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mol2", category: "scientific", container_id: None, recommended_ext: "mol2", accepted_exts: &["mol2"],
        patterns: &[MagicPattern::new(0, b"@<TRIPOS>MOLECULE")], min_header_len: 17, priority: 50,
        evidence_code: "rule.mol2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ase", category: "image", container_id: None, recommended_ext: "ase", accepted_exts: &["ase"],
        patterns: &[MagicPattern::new(0, b"ASEF")], min_header_len: 4, priority: 50,
        evidence_code: "rule.ase.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pat", category: "image", container_id: None, recommended_ext: "pat", accepted_exts: &["pat"],
        patterns: &[MagicPattern::new(0, b"8BPT")], min_header_len: 4, priority: 50,
        evidence_code: "rule.pat.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cpt", category: "image", container_id: None, recommended_ext: "cpt", accepted_exts: &["cpt"],
        patterns: &[MagicPattern::new(0, b"CPTFILE")], min_header_len: 7, priority: 50,
        evidence_code: "rule.cpt.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "afdesign", category: "image", container_id: None, recommended_ext: "afdesign", accepted_exts: &["afdesign"],
        patterns: &[MagicPattern::new(0, b"\x00\xff\x00\x00\x00\x00\x00\x00T3D")], min_header_len: 11, priority: 50,
        evidence_code: "rule.afdesign.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fh", category: "image", container_id: None, recommended_ext: "fh", accepted_exts: &["fh"],
        patterns: &[MagicPattern::new(0, b"AGGD")], min_header_len: 4, priority: 50,
        evidence_code: "rule.fh.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "art", category: "image", container_id: None, recommended_ext: "art", accepted_exts: &["art"],
        patterns: &[MagicPattern::new(0, b"JG\x04\x0e")], min_header_len: 4, priority: 50,
        evidence_code: "rule.art.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wpg", category: "image", container_id: None, recommended_ext: "wpg", accepted_exts: &["wpg"],
        patterns: &[MagicPattern::new(0, b"\xffWPC")], min_header_len: 4, priority: 50,
        evidence_code: "rule.wpg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cals", category: "image", container_id: None, recommended_ext: "cals", accepted_exts: &["cals", "cal"],
        patterns: &[MagicPattern::new(0, b"srcdocid:")], min_header_len: 9, priority: 50,
        evidence_code: "rule.cals.magic", verification: Verification::Signature, validator: None,
    },
];
