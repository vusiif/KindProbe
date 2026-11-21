use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

pub enum PngProbeResult {
    Confirmed { has_ihdr: bool },
    Truncated,
    None,
}

pub fn probe_png(header: &[u8]) -> PngProbeResult {
    if header.len() >= 8 && header.starts_with(PNG_SIGNATURE) {
        // Validate IHDR chunk (13 bytes data + 4 bytes 'IHDR')
        if header.len() >= 16 {
            let chunk_len = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
            let chunk_type = &header[12..16];
            if chunk_len == 13 && chunk_type == b"IHDR" {
                return PngProbeResult::Confirmed { has_ihdr: true };
            }
        }
        PngProbeResult::Confirmed { has_ihdr: false }
    } else if (header.len() >= 4 && header.starts_with(b"\x89PNG"))
        || (!header.is_empty() && header[0] == 0x89 && header.len() < 8)
    {
        PngProbeResult::Truncated
    } else {
        PngProbeResult::None
    }
}

pub fn make_png_report(has_ihdr: bool) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("png".to_string()),
        category: "image".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: if has_ihdr {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("png".to_string()),
        accepted_extensions: vec!["png".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: if has_ihdr {
            vec![
                EvidenceItem::new("sig.png.header"),
                EvidenceItem::new("sig.png.ihdr"),
            ]
        } else {
            vec![EvidenceItem::new("sig.png.header")]
        },
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
