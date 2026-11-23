use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static MODELS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "blend", category: "model", container_id: None, recommended_ext: "blend", accepted_exts: &["blend"],
        patterns: &[MagicPattern::new(0, b"BLENDER")],
        min_header_len: 7, priority: 50, evidence_code: "rule.blend.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "glb", category: "model", container_id: None, recommended_ext: "glb", accepted_exts: &["glb"],
        patterns: &[MagicPattern::new(0, b"glTF\x02\x00\x00\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.glb.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fbx", category: "model", container_id: None, recommended_ext: "fbx", accepted_exts: &["fbx"],
        patterns: &[MagicPattern::new(0, b"Kaydara FBX Binary  ")],
        min_header_len: 20, priority: 50, evidence_code: "rule.fbx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "3ds", category: "model", container_id: None, recommended_ext: "3ds", accepted_exts: &["3ds"],
        patterns: &[MagicPattern::new(0, b"MM\x02\x00")],
        min_header_len: 4, priority: 50, evidence_code: "rule.3ds.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ply", category: "model", container_id: None,
        recommended_ext: "ply", accepted_exts: &["ply"],
        patterns: &[MagicPattern::new(0, b"ply\x0a")],
        min_header_len: 4, priority: 50, evidence_code: "rule.ply.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "step", category: "model", container_id: None,
        recommended_ext: "stp", accepted_exts: &["stp", "step"],
        patterns: &[MagicPattern::new(0, b"ISO-10303-21;")],
        min_header_len: 13, priority: 50, evidence_code: "rule.step.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "lwo", category: "model", container_id: None,
        recommended_ext: "lwo", accepted_exts: &["lwo"],
        patterns: &[MagicPattern::new(0, b"FORM"), MagicPattern::new(8, b"LWO2")],
        min_header_len: 12, priority: 50, evidence_code: "rule.lwo.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bvh", category: "model", container_id: None,
        recommended_ext: "bvh", accepted_exts: &["bvh"],
        patterns: &[MagicPattern::new(0, b"HIERARCHY")],
        min_header_len: 9, priority: 50, evidence_code: "rule.bvh.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "usdc", category: "model", container_id: None, recommended_ext: "usdc", accepted_exts: &["usdc", "usd"],
        patterns: &[MagicPattern::new(0, b"PXR-USDC")], min_header_len: 8,
        priority: 50, evidence_code: "rule.usdc.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "u3d", category: "model", container_id: None, recommended_ext: "u3d", accepted_exts: &["u3d"],
        patterns: &[MagicPattern::new(0, b"U3D\x00")], min_header_len: 4,
        priority: 50, evidence_code: "rule.u3d.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "off", category: "model", container_id: None,
        recommended_ext: "off", accepted_exts: &["off"],
        patterns: &[MagicPattern::new(0, b"OFF\n")],
        min_header_len: 4, priority: 50, evidence_code: "rule.off.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ply", category: "model", container_id: None, recommended_ext: "ply",
        accepted_exts: &["ply"], patterns: &[MagicPattern::new(0, b"ply\n")],
        min_header_len: 4, priority: 50, evidence_code: "rule.ply.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "3ds", category: "model", container_id: None, recommended_ext: "3ds",
        accepted_exts: &["3ds"], patterns: &[MagicPattern::new(0, b"\x4d\x4d")],
        min_header_len: 2, priority: 50, evidence_code: "rule.3ds.magic", verification: Verification::Signature, validator: None,
    },
];
