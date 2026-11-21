use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const RAR4_SIGNATURE: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
pub const RAR5_SIGNATURE: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];

pub enum RarProbeResult {
    ConfirmedRar4 { has_main_header: bool },
    ConfirmedRar5,
    Truncated,
    None,
}

pub fn probe_rar(header: &[u8]) -> RarProbeResult {
    // RAR 5.0+ signature (8 bytes)
    if header.len() >= 8 && header.starts_with(RAR5_SIGNATURE) {
        return RarProbeResult::ConfirmedRar5;
    }

    // RAR 1.5 - 4.x signature (7 bytes)
    if header.len() >= 7 && header.starts_with(RAR4_SIGNATURE) {
        let mut has_main_header = false;
        // In RAR 4, the marker block is followed by the Main Archive Header (type 0x73)
        // Offset 7..9: HEAD_CRC (2 bytes)
        // Offset 9: HEAD_TYPE (1 byte, 0x73 = MAIN_HEAD)
        if header.len() >= 10 && header[9] == 0x73 {
            has_main_header = true;
        }
        return RarProbeResult::ConfirmedRar4 { has_main_header };
    }

    // Truncated detection
    if !header.is_empty() && header.len() < 8 && (RAR4_SIGNATURE.starts_with(header) || RAR5_SIGNATURE.starts_with(header)) {
        RarProbeResult::Truncated
    } else {
        RarProbeResult::None
    }
}

pub fn make_rar4_report(has_main_header: bool) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("rar".to_string()),
        category: "archive".to_string(),
        container_id: Some("rar".to_string()),
        identification: Identification::Format,
        verification: if has_main_header {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("rar".to_string()),
        accepted_extensions: vec!["rar".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: if has_main_header {
            vec![
                EvidenceItem::new("sig.rar.header"),
                EvidenceItem::new("sig.rar.main_header"),
            ]
        } else {
            vec![EvidenceItem::new("sig.rar.header")]
        },
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}

pub fn make_rar5_report() -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("rar".to_string()),
        category: "archive".to_string(),
        container_id: Some("rar".to_string()),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("rar".to_string()),
        accepted_extensions: vec!["rar".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![
            EvidenceItem::new("sig.rar.header"),
            EvidenceItem::new("sig.rar5.signature"),
        ],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
