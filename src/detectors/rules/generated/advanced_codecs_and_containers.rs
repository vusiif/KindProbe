use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static ADVANCED_CODECS_AND_CONTAINERS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "cuneiform_fed", category: "document", container_id: None, recommended_ext: "fed", accepted_exts: &["fed"],
        patterns: &[MagicPattern::new(0, b"CUNEIFORM\x00")], min_header_len: 10, priority: 50,
        evidence_code: "rule.cuneiform_fed.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "nvidia_ptx", category: "code", container_id: None, recommended_ext: "ptx", accepted_exts: &["ptx"],
        patterns: &[MagicPattern::new(0, b".version 7.")], min_header_len: 11, priority: 50,
        evidence_code: "rule.nvidia_ptx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "spirv_shader", category: "code", container_id: None, recommended_ext: "spv", accepted_exts: &["spv"],
        patterns: &[MagicPattern::new(0, b"\x03\x02\x23\x07")], min_header_len: 4, priority: 50,
        evidence_code: "rule.spirv_shader.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dvd_bup", category: "video", container_id: None, recommended_ext: "bup", accepted_exts: &["bup"],
        patterns: &[MagicPattern::new(0, b"DVDVIDEO-VMG")], min_header_len: 12, priority: 50,
        evidence_code: "rule.dvd_bup.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "canon_cr3", category: "image", container_id: None, recommended_ext: "cr3", accepted_exts: &["cr3"],
        patterns: &[MagicPattern::new(4, b"ftypcrx ")], min_header_len: 12, priority: 50,
        evidence_code: "rule.canon_cr3.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "panasonic_rw2", category: "image", container_id: None, recommended_ext: "rw2", accepted_exts: &["rw2"],
        patterns: &[MagicPattern::new(0, b"IIU\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.panasonic_rw2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "olympus_orf", category: "image", container_id: None, recommended_ext: "orf", accepted_exts: &["orf"],
        patterns: &[MagicPattern::new(0, b"IIRO\x08\x00\x00\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.olympus_orf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "geopackage", category: "database", container_id: None, recommended_ext: "gpkg", accepted_exts: &["gpkg"],
        patterns: &[MagicPattern::new(68, b"GPKG")], min_header_len: 72, priority: 50,
        evidence_code: "rule.geopackage.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "internet_archive_arc", category: "archive", container_id: None, recommended_ext: "arc_ia", accepted_exts: &["arc_ia"],
        patterns: &[MagicPattern::new(0, b"filedesc://")], min_header_len: 11, priority: 50,
        evidence_code: "rule.internet_archive_arc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mobipocket", category: "ebook", container_id: None, recommended_ext: "mobi", accepted_exts: &["mobi"],
        patterns: &[MagicPattern::new(60, b"BOOKMOBI")], min_header_len: 68, priority: 50,
        evidence_code: "rule.mobipocket.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "zstd_dict", category: "archive", container_id: None, recommended_ext: "zstdict", accepted_exts: &["zstdict"],
        patterns: &[MagicPattern::new(0, b"\x37\xa4\x30\xec")], min_header_len: 4, priority: 50,
        evidence_code: "rule.zstd_dict.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cramfs_img", category: "disk", container_id: None, recommended_ext: "cramfs", accepted_exts: &["cramfs"],
        patterns: &[MagicPattern::new(0, b"\x28\xcd\x3d\x45")], min_header_len: 4, priority: 50,
        evidence_code: "rule.cramfs_img.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bzip3_archive", category: "archive", container_id: None, recommended_ext: "bz3", accepted_exts: &["bz3"],
        patterns: &[MagicPattern::new(0, b"BZ3v1")], min_header_len: 5, priority: 50,
        evidence_code: "rule.bzip3_archive.magic", verification: Verification::Signature, validator: None,
    },
];
