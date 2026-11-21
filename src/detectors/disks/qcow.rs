use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const QCOW_MAGIC: &[u8] = &[0x51, 0x46, 0x49, 0xFB]; // "QFI\xFB"

pub enum QcowMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes QEMU Copy-On-Write (QCOW / QCOW2) disk image format.
pub fn probe_qcow(header: &[u8]) -> QcowMatch {
    if header.len() < 4 {
        if !header.is_empty() && header.starts_with(b"QF") {
            return QcowMatch::Truncated("qcow2".to_string());
        }
        return QcowMatch::None;
    }

    if !header.starts_with(QCOW_MAGIC) {
        return QcowMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.qcow.magic")];

    let verification = if header.len() >= 24 {
        let version = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
        let cluster_bits = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);

        if (version == 2 || version == 3) && (9..=21).contains(&cluster_bits) {
            evidence.push(EvidenceItem::new("sig.qcow.header"));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    QcowMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("qcow2".to_string()),
        category: "disk".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("qcow2".to_string()),
        accepted_extensions: vec![
            "qcow2".to_string(),
            "qcow".to_string(),
            "img".to_string(),
        ],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
