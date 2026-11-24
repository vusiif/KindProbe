use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static OPTICAL_DISC_MASTERING_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "clonecd_ccd", category: "disk_image", container_id: None, recommended_ext: "ccd", accepted_exts: &["ccd"],
        patterns: &[MagicPattern::new(0, b"[CloneCD]\nVersion=")], min_header_len: 18, priority: 50,
        evidence_code: "rule.clonecd_ccd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "alcohol_mds", category: "disk_image", container_id: None, recommended_ext: "mds", accepted_exts: &["mds"],
        patterns: &[MagicPattern::new(0, b"MEDIA DESCRIPTOR")], min_header_len: 16, priority: 50,
        evidence_code: "rule.alcohol_mds.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "nero_nrg2", category: "disk_image", container_id: None, recommended_ext: "nrg2", accepted_exts: &["nrg2"],
        patterns: &[MagicPattern::new(0, b"NER2\x00\x00\x00\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.nero_nrg2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "blindwrite_b5t", category: "disk_image", container_id: None, recommended_ext: "b5t", accepted_exts: &["b5t"],
        patterns: &[MagicPattern::new(0, b"B5T_HEADER_V1")], min_header_len: 13, priority: 50,
        evidence_code: "rule.blindwrite_b5t.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "blindwrite_b6t", category: "disk_image", container_id: None, recommended_ext: "b6t", accepted_exts: &["b6t"],
        patterns: &[MagicPattern::new(0, b"B6T_HEADER_V1")], min_header_len: 13, priority: 50,
        evidence_code: "rule.blindwrite_b6t.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "discjuggler_cdi", category: "disk_image", container_id: None, recommended_ext: "cdi", accepted_exts: &["cdi"],
        patterns: &[MagicPattern::new(0, b"DISCJUGGLER_CDI_V2\x00")], min_header_len: 19, priority: 50,
        evidence_code: "rule.discjuggler_cdi.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cdrwin_cue", category: "disk_image", container_id: None, recommended_ext: "cue_cdrwin", accepted_exts: &["cue_cdrwin"],
        patterns: &[MagicPattern::new(0, b"FILE \"\" BINARY\n  TRACK 01 MODE1/2352\n")], min_header_len: 39, priority: 50,
        evidence_code: "rule.cdrwin_cue.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cdrdao_toc", category: "disk_image", container_id: None, recommended_ext: "toc_cdrdao", accepted_exts: &["toc_cdrdao"],
        patterns: &[MagicPattern::new(0, b"CD_DA\n\nTRACK AUDIO\n")], min_header_len: 19, priority: 50,
        evidence_code: "rule.cdrdao_toc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gear_mastering_p01", category: "disk_image", container_id: None, recommended_ext: "p01", accepted_exts: &["p01"],
        patterns: &[MagicPattern::new(0, b"GEAR_OPTICAL_IMAGE\x00")], min_header_len: 19, priority: 50,
        evidence_code: "rule.gear_mastering_p01.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "plex_tools_pxi", category: "disk_image", container_id: None, recommended_ext: "pxi", accepted_exts: &["pxi"],
        patterns: &[MagicPattern::new(0, b"PLEXTOOLS_IMAGE\x00")], min_header_len: 16, priority: 50,
        evidence_code: "rule.plex_tools_pxi.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "virtual_cd_vc4", category: "disk_image", container_id: None, recommended_ext: "vc4", accepted_exts: &["vc4"],
        patterns: &[MagicPattern::new(0, b"VirtualCD_V4_Image\x00")], min_header_len: 19, priority: 50,
        evidence_code: "rule.virtual_cd_vc4.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gamejack_xmd", category: "disk_image", container_id: None, recommended_ext: "xmd", accepted_exts: &["xmd"],
        patterns: &[MagicPattern::new(0, b"XMD_DISC_IMAGE\x00")], min_header_len: 15, priority: 50,
        evidence_code: "rule.gamejack_xmd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "padus_cdi_v3", category: "disk_image", container_id: None, recommended_ext: "cdi3", accepted_exts: &["cdi3"],
        patterns: &[MagicPattern::new(0, b"CDI_V3_PADUS_IMAGE\x00")], min_header_len: 19, priority: 50,
        evidence_code: "rule.padus_cdi_v3.magic", verification: Verification::Signature, validator: None,
    },
];
