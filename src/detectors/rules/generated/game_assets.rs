use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static GAME_ASSETS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "wad", category: "archive", container_id: None,
        recommended_ext: "wad", accepted_exts: &["wad"],
        patterns: &[MagicPattern::new(0, b"IWAD")],
        min_header_len: 4, priority: 50, evidence_code: "rule.wad.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wad", category: "archive", container_id: None,
        recommended_ext: "wad", accepted_exts: &["wad"],
        patterns: &[MagicPattern::new(0, b"PWAD")],
        min_header_len: 4, priority: 50, evidence_code: "rule.wad.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pak", category: "archive", container_id: None,
        recommended_ext: "pak", accepted_exts: &["pak"],
        patterns: &[MagicPattern::new(0, b"PACK")],
        min_header_len: 4, priority: 50, evidence_code: "rule.pak.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pck", category: "archive", container_id: None,
        recommended_ext: "pck", accepted_exts: &["pck"],
        patterns: &[MagicPattern::new(0, b"GDPC")],
        min_header_len: 4, priority: 50, evidence_code: "rule.pck.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "unity3d", category: "archive", container_id: None,
        recommended_ext: "unity3d", accepted_exts: &["unity3d", "bundle", "asset"],
        patterns: &[MagicPattern::new(0, b"UnityFS")],
        min_header_len: 7, priority: 50, evidence_code: "rule.unity3d.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vpk", category: "archive", container_id: None,
        recommended_ext: "vpk", accepted_exts: &["vpk"],
        patterns: &[MagicPattern::new(0, b"\x34\x12\xaa\x55")],
        min_header_len: 4, priority: 50, evidence_code: "rule.vpk.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "big", category: "archive", container_id: None, recommended_ext: "big", accepted_exts: &["big"],
        patterns: &[MagicPattern::new(0, b"BIG4")], min_header_len: 4,
        priority: 50, evidence_code: "rule.big.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "grp", category: "archive", container_id: None, recommended_ext: "grp", accepted_exts: &["grp"],
        patterns: &[MagicPattern::new(0, b"KenSilverman")], min_header_len: 12,
        priority: 50, evidence_code: "rule.grp.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gen", category: "rom", container_id: None,
        recommended_ext: "gen", accepted_exts: &["gen", "smd", "md"],
        patterns: &[MagicPattern::new(256, b"SEGA")],
        min_header_len: 260, priority: 50, evidence_code: "rule.gen.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "voc", category: "audio", container_id: None,
        recommended_ext: "voc", accepted_exts: &["voc"],
        patterns: &[MagicPattern::new(0, b"Creative Voice File\x1a")],
        min_header_len: 20, priority: 50, evidence_code: "rule.voc.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bik", category: "video", container_id: None,
        recommended_ext: "bik", accepted_exts: &["bik", "bk2"],
        patterns: &[MagicPattern::new(0, b"BIK")],
        min_header_len: 3, priority: 50, evidence_code: "rule.bik.magic",
        verification: Verification::Signature, validator: None,
    },
];
