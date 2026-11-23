use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static EMBEDDED_RTOS_IMAGES_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "uf2", category: "firmware", container_id: None, recommended_ext: "uf2", accepted_exts: &["uf2"],
        patterns: &[MagicPattern::new(0, b"\x55\x46\x32\x0a\x57\x51\x5d\x9e")], min_header_len: 8, priority: 50,
        evidence_code: "rule.uf2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "esp32_img", category: "firmware", container_id: None, recommended_ext: "esp32", accepted_exts: &["esp32"],
        patterns: &[MagicPattern::new(0, b"\xe9\x01\x02\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.esp32_img.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "itb", category: "firmware", container_id: None, recommended_ext: "itb", accepted_exts: &["itb"],
        patterns: &[MagicPattern::new(0, b"\xd0\x0d\xfe\xed\x00\x00\x01")], min_header_len: 7, priority: 48,
        evidence_code: "rule.itb.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "qnx_ifs", category: "firmware", container_id: None, recommended_ext: "qnx", accepted_exts: &["qnx"],
        patterns: &[MagicPattern::new(0, b"\xeb\x7e\xff\x00\x01")], min_header_len: 5, priority: 48,
        evidence_code: "rule.qnx_ifs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ti_txt", category: "firmware", container_id: None, recommended_ext: "titxt", accepted_exts: &["titxt"],
        patterns: &[MagicPattern::new(0, b"@F000\r\n")], min_header_len: 7, priority: 50,
        evidence_code: "rule.ti_txt.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ubifs", category: "firmware", container_id: None, recommended_ext: "ubi", accepted_exts: &["ubi", "ubifs"],
        patterns: &[MagicPattern::new(0, b"UBI#")], min_header_len: 4, priority: 50,
        evidence_code: "rule.ubifs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bcm_trx", category: "firmware", container_id: None, recommended_ext: "trx2", accepted_exts: &["trx2"],
        patterns: &[MagicPattern::new(0, b"HDR1")], min_header_len: 4, priority: 50,
        evidence_code: "rule.bcm_trx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dlink_fw", category: "firmware", container_id: None, recommended_ext: "dlk", accepted_exts: &["dlk"],
        patterns: &[MagicPattern::new(0, b"SHRS")], min_header_len: 4, priority: 50,
        evidence_code: "rule.dlink_fw.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "netgear_fw", category: "firmware", container_id: None, recommended_ext: "chk", accepted_exts: &["chk"],
        patterns: &[MagicPattern::new(0, b"2.00\x00\x00\x00\x00")], min_header_len: 8, priority: 45,
        evidence_code: "rule.netgear_fw.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "openwrt_trx", category: "firmware", container_id: None, recommended_ext: "owrt", accepted_exts: &["owrt"],
        patterns: &[MagicPattern::new(0, b"OWRT")], min_header_len: 4, priority: 50,
        evidence_code: "rule.openwrt_trx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "jlink_hex", category: "firmware", container_id: None, recommended_ext: "jlink", accepted_exts: &["jlink"],
        patterns: &[MagicPattern::new(0, b"SEGGER")], min_header_len: 6, priority: 50,
        evidence_code: "rule.jlink_hex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "allwinner_egon", category: "firmware", container_id: None, recommended_ext: "egon", accepted_exts: &["egon"],
        patterns: &[MagicPattern::new(4, b"eGON.BT0")], min_header_len: 12, priority: 50,
        evidence_code: "rule.allwinner_egon.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "intel_conf", category: "firmware", container_id: None, recommended_ext: "ifd", accepted_exts: &["ifd"],
        patterns: &[MagicPattern::new(16, b"\x5a\xa5\xf0\x0f")], min_header_len: 20, priority: 50,
        evidence_code: "rule.intel_conf.magic", verification: Verification::Signature, validator: None,
    },
];
