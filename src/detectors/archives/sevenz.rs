use crate::detectors::archives::crc32::crc32_ieee;
use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub const SEVENZ_SIGNATURE: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

pub enum SevenZProbeResult {
    Confirmed { has_start_header: bool },
    Truncated,
    None,
}

pub fn probe_sevenz(header: &[u8]) -> SevenZProbeResult {
    if header.len() >= 6 && header.starts_with(SEVENZ_SIGNATURE) {
        let mut has_start_header = false;
        // 7z StartHeader verification:
        // Offset 0..6: Signature (6 bytes)
        // Offset 6..8: ArchiveVersion (2 bytes: major, minor)
        // Offset 8..12: StartHeaderCRC (4 bytes, CRC32 of bytes 12..32)
        // Offset 12..32: NextHeaderOffset (8 bytes) + NextHeaderSize (8 bytes) + NextHeaderCRC (4 bytes)
        if header.len() >= 32 {
            let expected_crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            let actual_crc = crc32_ieee(&header[12..32]);
            if expected_crc == actual_crc {
                has_start_header = true;
            }
        }

        SevenZProbeResult::Confirmed { has_start_header }
    } else if header.len() >= 2 && header.len() < 6 && SEVENZ_SIGNATURE.starts_with(header) {
        SevenZProbeResult::Truncated
    } else {
        SevenZProbeResult::None
    }
}

pub fn make_sevenz_core(has_start_header: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("7z"),
        category: "archive",
        container_id: Some("7z"),
        identification: Identification::Format,
        verification: if has_start_header {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("7z"),
        accepted_extensions: &["7z"],
        primary_evidence: Some("sig.7z.header"),
        secondary_evidence: if has_start_header {
            Some("sig.7z.start_header")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_sevenz_report(has_start_header: bool) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("7z".to_string()),
        category: "archive".to_string(),
        container_id: Some("7z".to_string()),
        identification: Identification::Format,
        verification: if has_start_header {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("7z".to_string()),
        accepted_extensions: vec!["7z".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: if has_start_header {
            vec![
                EvidenceItem::new("sig.7z.header"),
                EvidenceItem::new("sig.7z.start_header"),
            ]
        } else {
            vec![EvidenceItem::new("sig.7z.header")]
        },
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
