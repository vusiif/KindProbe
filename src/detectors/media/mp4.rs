use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum Mp4Match {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes ISO-BMFF box structure for MP4 and QuickTime MOV.
pub fn probe_mp4(header: &[u8]) -> Mp4Match {
    // 1. Truncation checks
    if header.len() < 8 {
        if header.len() >= 4 && (header.starts_with(&[0, 0, 0]) || header.starts_with(b"ftyp")) {
            return Mp4Match::Truncated("mp4".to_string());
        }
        return Mp4Match::None;
    }

    let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let box_type = &header[4..8];

    // Check for ftyp box
    if box_type == b"ftyp" {
        if box_size < 16 && box_size != 1 {
            return Mp4Match::None;
        }

        // Header must be at least 12 bytes to inspect major brand
        if header.len() < 12 {
            return Mp4Match::Truncated("mp4".to_string());
        }

        let major_brand = &header[8..12];
        let mut evidence = vec![EvidenceItem::new("sig.mp4.ftyp")];

        // Check if QuickTime MOV: major_brand == b"qt  "
        if major_brand == b"qt  " {
            evidence.push(EvidenceItem::new("sig.mov.brand"));
            return Mp4Match::Confirmed(Box::new(DetectionReport {
                schema_version: 1,
                format_id: Some("mov".to_string()),
                category: "video".to_string(),
                container_id: Some("isobmff".to_string()),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("mov".to_string()),
                accepted_extensions: vec!["mov".to_string(), "qt".to_string()],
                extension_status: ExtensionStatus::Undetermined,
                evidence,
                warnings: Vec::new(),
                stats: Default::default(),
                candidates: None,
            }));
        }

        evidence.push(EvidenceItem::new("sig.mp4.brand"));
        let (format_id, accepted_exts, rec_ext) = if major_brand == b"M4A " {
            ("m4a", vec!["m4a".to_string(), "mp4".to_string()], "m4a")
        } else {
            ("mp4", vec!["mp4".to_string(), "m4v".to_string(), "m4a".to_string()], "mp4")
        };

        return Mp4Match::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some(format_id.to_string()),
            category: if format_id == "m4a" {
                "audio".to_string()
            } else {
                "video".to_string()
            },
            container_id: Some("isobmff".to_string()),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some(rec_ext.to_string()),
            accepted_extensions: accepted_exts,
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // Legacy QuickTime MOV starting with moov, wide, or free at file offset 0
    if box_type == b"moov" || box_type == b"wide" || box_type == b"free" {
        return Mp4Match::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("mov".to_string()),
            category: "video".to_string(),
            container_id: Some("isobmff".to_string()),
            identification: Identification::Format,
            verification: Verification::Signature,
            inspection: Inspection::Complete,
            recommended_extension: Some("mov".to_string()),
            accepted_extensions: vec!["mov".to_string(), "qt".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.mov.header")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    Mp4Match::None
}
