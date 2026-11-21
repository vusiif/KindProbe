use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum JpegProbeResult {
    Confirmed { has_segment: bool },
    Truncated,
    None,
}

pub fn probe_jpeg(header: &[u8]) -> JpegProbeResult {
    if header.len() >= 3 && header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF {
        // Validate following marker segment
        let mut has_segment = false;
        if header.len() >= 4 {
            let marker = header[3];
            // Valid JPEG marker after SOI:
            // 0xE0..=0xEF (APP0-APP15), 0xDB (DQT), 0xC0..=0xC3 (SOF0-SOF3), 0xFE (COM)
            if (0xE0..=0xEF).contains(&marker) || marker == 0xDB || (0xC0..=0xC3).contains(&marker) || marker == 0xFE {
                // If we have at least 6 bytes, we can verify segment length
                if header.len() >= 6 {
                    let seg_len = u16::from_be_bytes([header[4], header[5]]) as usize;
                    // Segment length field includes the 2 bytes of the length itself (must be >= 2)
                    if seg_len >= 2 {
                        has_segment = true;
                    }
                } else {
                    has_segment = true;
                }
            }
        }

        JpegProbeResult::Confirmed { has_segment }
    } else if (header.len() >= 2 && header[0] == 0xFF && header[1] == 0xD8 && header.len() < 3)
        || (header.len() == 1 && header[0] == 0xFF)
    {
        JpegProbeResult::Truncated
    } else {
        JpegProbeResult::None
    }
}

pub fn make_jpeg_report(has_segment: bool) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("jpeg".to_string()),
        category: "image".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: if has_segment {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("jpg".to_string()),
        accepted_extensions: vec![
            "jpg".to_string(),
            "jpeg".to_string(),
            "jfif".to_string(),
            "jpe".to_string(),
        ],
        extension_status: ExtensionStatus::Undetermined,
        evidence: if has_segment {
            vec![
                EvidenceItem::new("sig.jpeg.soi"),
                EvidenceItem::new("sig.jpeg.segment"),
            ]
        } else {
            vec![EvidenceItem::new("sig.jpeg.soi")]
        },
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
