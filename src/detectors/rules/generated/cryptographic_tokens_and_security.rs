use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static CRYPTOGRAPHIC_TOKENS_AND_SECURITY_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "pkcs7_pem", category: "crypto", container_id: None, recommended_ext: "p7b", accepted_exts: &["p7b"],
        patterns: &[MagicPattern::new(0, b"-----BEGIN PKCS7-----")], min_header_len: 21, priority: 50,
        evidence_code: "rule.pkcs7_pem.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "spkac", category: "crypto", container_id: None, recommended_ext: "spkac", accepted_exts: &["spkac"],
        patterns: &[MagicPattern::new(0, b"SPKAC=")], min_header_len: 6, priority: 50,
        evidence_code: "rule.spkac.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "tss2_key", category: "crypto", container_id: None, recommended_ext: "tss2", accepted_exts: &["tss2"],
        patterns: &[MagicPattern::new(0, b"-----BEGIN TSS2 PRIVATE KEY-----")], min_header_len: 32, priority: 50,
        evidence_code: "rule.tss2_key.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pkcs15_profile", category: "crypto", container_id: None, recommended_ext: "p15", accepted_exts: &["p15"],
        patterns: &[MagicPattern::new(0, b"PKCS#15 Token Profile")], min_header_len: 22, priority: 50,
        evidence_code: "rule.pkcs15_profile.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wireguard_conf", category: "crypto", container_id: None, recommended_ext: "conf", accepted_exts: &["conf"],
        patterns: &[MagicPattern::new(0, b"[Interface]\nPrivateKey = ")], min_header_len: 25, priority: 50,
        evidence_code: "rule.wireguard_conf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "openvpn_conf", category: "crypto", container_id: None, recommended_ext: "ovpn", accepted_exts: &["ovpn"],
        patterns: &[MagicPattern::new(0, b"client\ndev tun\n")], min_header_len: 15, priority: 50,
        evidence_code: "rule.openvpn_conf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "clamav_cvd", category: "security", container_id: None, recommended_ext: "cvd", accepted_exts: &["cvd"],
        patterns: &[MagicPattern::new(0, b"ClamAV-VDB:")], min_header_len: 11, priority: 50,
        evidence_code: "rule.clamav_cvd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "tpm2_key", category: "crypto", container_id: None, recommended_ext: "tpmkey", accepted_exts: &["tpmkey"],
        patterns: &[MagicPattern::new(0, b"TPM2_KEY")], min_header_len: 8, priority: 50,
        evidence_code: "rule.tpm2_key.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "yubikey_oath", category: "crypto", container_id: None, recommended_ext: "ykoath", accepted_exts: &["ykoath"],
        patterns: &[MagicPattern::new(0, b"YUBICO_OATH_V1")], min_header_len: 14, priority: 50,
        evidence_code: "rule.yubikey_oath.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "age_identity", category: "crypto", container_id: None, recommended_ext: "key", accepted_exts: &["key"],
        patterns: &[MagicPattern::new(0, b"# created by age-keygen\nAGE-SECRET-KEY-1")], min_header_len: 40, priority: 50,
        evidence_code: "rule.age_identity.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bitwarden_export", category: "crypto", container_id: None, recommended_ext: "json", accepted_exts: &["json"],
        patterns: &[MagicPattern::new(0, b"{\"encrypted\":true")], min_header_len: 17, priority: 50,
        evidence_code: "rule.bitwarden_export.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "keepassx_xml", category: "crypto", container_id: None, recommended_ext: "xml", accepted_exts: &["xml"],
        patterns: &[MagicPattern::new(0, b"<!DOCTYPE KEEPASSX_DATABASE>")], min_header_len: 28, priority: 50,
        evidence_code: "rule.keepassx_xml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "android_adb_backup", category: "archive", container_id: None, recommended_ext: "ab", accepted_exts: &["ab"],
        patterns: &[MagicPattern::new(0, b"ANDROID BACKUP\n")], min_header_len: 15, priority: 50,
        evidence_code: "rule.android_adb_backup.magic", verification: Verification::Signature, validator: None,
    },
];
