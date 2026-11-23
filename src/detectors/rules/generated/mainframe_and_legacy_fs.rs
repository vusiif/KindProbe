use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static MAINFRAME_AND_LEGACY_FS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "het", category: "virtualization", container_id: None, recommended_ext: "het", accepted_exts: &["het"],
        patterns: &[MagicPattern::new(0, b"HET\x01")], min_header_len: 4, priority: 50,
        evidence_code: "rule.het.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "atr", category: "disk_image", container_id: None, recommended_ext: "atr", accepted_exts: &["atr"],
        patterns: &[MagicPattern::new(0, b"\x96\x02")], min_header_len: 2, priority: 50,
        evidence_code: "rule.atr.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "crt_c64", category: "rom", container_id: None, recommended_ext: "crt", accepted_exts: &["crt"],
        patterns: &[MagicPattern::new(0, b"C64 CARTRIDGE   ")], min_header_len: 16, priority: 50,
        evidence_code: "rule.crt_c64.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "xdf", category: "disk_image", container_id: None, recommended_ext: "xdf", accepted_exts: &["xdf"],
        patterns: &[MagicPattern::new(0, b"XDF\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.xdf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dim", category: "disk_image", container_id: None, recommended_ext: "dim", accepted_exts: &["dim"],
        patterns: &[MagicPattern::new(0, b"DIFC")], min_header_len: 4, priority: 50,
        evidence_code: "rule.dim.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "nfd", category: "disk_image", container_id: None, recommended_ext: "nfd", accepted_exts: &["nfd"],
        patterns: &[MagicPattern::new(0, b"T98FDDIMAGE.R0")], min_header_len: 14, priority: 50,
        evidence_code: "rule.nfd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bkd", category: "disk_image", container_id: None, recommended_ext: "bkd", accepted_exts: &["bkd"],
        patterns: &[MagicPattern::new(0, b"BK0010")], min_header_len: 6, priority: 50,
        evidence_code: "rule.bkd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dsk_cpc", category: "disk_image", container_id: None, recommended_ext: "dsk", accepted_exts: &["dsk"],
        patterns: &[MagicPattern::new(0, b"EXTENDED CPC DSK File\r\nDisk-Info\r\n")], min_header_len: 34, priority: 50,
        evidence_code: "rule.dsk_cpc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ipf", category: "disk_image", container_id: None, recommended_ext: "ipf", accepted_exts: &["ipf"],
        patterns: &[MagicPattern::new(0, b"CAPS\x00\x00\x00\x0c")], min_header_len: 8, priority: 50,
        evidence_code: "rule.ipf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "stx", category: "disk_image", container_id: None, recommended_ext: "stx", accepted_exts: &["stx"],
        patterns: &[MagicPattern::new(0, b"RSY\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.stx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "sad", category: "disk_image", container_id: None, recommended_ext: "sad", accepted_exts: &["sad"],
        patterns: &[MagicPattern::new(0, b"Aley's disk backup")], min_header_len: 18, priority: 50,
        evidence_code: "rule.sad.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cpm_imd", category: "disk_image", container_id: None, recommended_ext: "imd", accepted_exts: &["imd"],
        patterns: &[MagicPattern::new(0, b"IMD ")], min_header_len: 4, priority: 50,
        evidence_code: "rule.cpm_imd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "hdi", category: "disk_image", container_id: None, recommended_ext: "hdi", accepted_exts: &["hdi"],
        patterns: &[MagicPattern::new(0, b"\x00\x00\x00\x00\x20\x00\x00\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.hdi.magic", verification: Verification::Signature, validator: None,
    },
];
