use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub const QCOW_MAGIC: &[u8] = &[0x51, 0x46, 0x49, 0xFB]; // "QFI\xFB"

pub enum QcowMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_qcow2_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("qcow2"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("qcow2"),
        accepted_extensions: &["qcow2", "qcow", "img"],
        primary_evidence: Some("sig.qcow.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.qcow.header")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_qcow_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 4 || !header.starts_with(QCOW_MAGIC) {
        return None;
    }

    let verification = if header.len() >= 24 {
        let version = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
        let cluster_bits = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);

        if (version == 2 || version == 3) && (9..=21).contains(&cluster_bits) {
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    Some(make_qcow2_core(verification))
}

/// Probes QEMU Copy-On-Write (QCOW / QCOW2) disk image format.
pub fn probe_qcow(header: &[u8]) -> QcowMatch {
    if header.len() < 4 {
        if !header.is_empty() && header.starts_with(b"QF") {
            return QcowMatch::Truncated("qcow2".to_string());
        }
        return QcowMatch::None;
    }

    match probe_qcow_core(header) {
        Some(core) => QcowMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => QcowMatch::None,
    }
}
