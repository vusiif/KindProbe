use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum GifProbeResult {
    Confirmed { has_screen_desc: bool },
    Truncated,
    None,
}

pub fn probe_gif(header: &[u8]) -> GifProbeResult {
    if header.len() >= 6 && (header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a")) {
        let mut has_screen_desc = false;
        if header.len() >= 10 {
            let width = u16::from_le_bytes([header[6], header[7]]);
            let height = u16::from_le_bytes([header[8], header[9]]);
            if width > 0 && height > 0 {
                has_screen_desc = true;
            }
        }
        GifProbeResult::Confirmed { has_screen_desc }
    } else if (header.len() >= 3 && header.starts_with(b"GIF") && header.len() < 6)
        || (header.len() >= 1 && header[0] == b'G' && header.len() < 3)
    {
        GifProbeResult::Truncated
    } else {
        GifProbeResult::None
    }
}

pub fn make_gif_report(has_screen_desc: bool) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("gif".to_string()),
        category: "image".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: if has_screen_desc {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("gif".to_string()),
        accepted_extensions: vec!["gif".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: if has_screen_desc {
            vec![
                EvidenceItem::new("sig.gif.header"),
                EvidenceItem::new("sig.gif.logical_screen"),
            ]
        } else {
            vec![EvidenceItem::new("sig.gif.header")]
        },
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
