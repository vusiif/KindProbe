use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub const VMDK_SPARSE_MAGIC: &[u8] = b"KDMV";
pub const VMDK_DESC_MAGIC: &[u8] = b"# Disk DescriptorFile";

pub enum VmdkMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_vmdk_sparse_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("vmdk"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("vmdk"),
        accepted_extensions: &["vmdk"],
        primary_evidence: Some("sig.vmdk.sparse_magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.vmdk.version")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_vmdk_desc_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("vmdk"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("vmdk"),
        accepted_extensions: &["vmdk"],
        primary_evidence: Some("sig.vmdk.desc_magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_vmdk_core(header: &[u8]) -> Option<DetectionCore> {
    if header.starts_with(VMDK_SPARSE_MAGIC) {
        let verification = if header.len() >= 8 {
            let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
            if (1..=3).contains(&version) {
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };

        return Some(make_vmdk_sparse_core(verification));
    }

    if header.starts_with(VMDK_DESC_MAGIC) {
        return Some(make_vmdk_desc_core());
    }

    None
}

/// Probes VMware Virtual Disk (VMDK) format.
pub fn probe_vmdk(header: &[u8]) -> VmdkMatch {
    if header.len() < 4 {
        if header.len() >= 2 && header.starts_with(b"KD") {
            return VmdkMatch::Truncated("vmdk".to_string());
        }
        return VmdkMatch::None;
    }

    match probe_vmdk_core(header) {
        Some(core) => VmdkMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => VmdkMatch::None,
    }
}
