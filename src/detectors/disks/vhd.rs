use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub const VHDX_MAGIC: &[u8] = b"vhdxfile";
pub const VHD_MAGIC: &[u8] = b"conectix";

pub enum VhdMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_vhdx_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("vhdx"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("vhdx"),
        accepted_extensions: &["vhdx"],
        primary_evidence: Some("sig.vhdx.magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_vhd_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("vhd"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("vhd"),
        accepted_extensions: &["vhd"],
        primary_evidence: Some("sig.vhd.magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_vhd_core(header: &[u8]) -> Option<DetectionCore> {
    if header.starts_with(VHDX_MAGIC) {
        return Some(make_vhdx_core());
    }
    if header.starts_with(VHD_MAGIC) {
        return Some(make_vhd_core());
    }
    None
}

/// Probes Microsoft Virtual Hard Disk (VHD & VHDX) formats.
pub fn probe_vhd(header: &[u8]) -> VhdMatch {
    if header.len() < 8 {
        if !header.is_empty() && (header.starts_with(b"vhdx") || header.starts_with(b"cone")) {
            return VhdMatch::Truncated(if header.starts_with(b"vhdx") {
                "vhdx".to_string()
            } else {
                "vhd".to_string()
            });
        }
        return VhdMatch::None;
    }

    match probe_vhd_core(header) {
        Some(core) => VhdMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => VhdMatch::None,
    }
}
