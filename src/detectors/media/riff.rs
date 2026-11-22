use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum RiffMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes RIFF container and parses WAVE / WEBP / AVI subtypes.
pub fn probe_riff(header: &[u8]) -> RiffMatch {
    if header.len() < 4 {
        if header.len() >= 2 && header.starts_with(b"RI") {
            return RiffMatch::Truncated("wav".to_string());
        }
        return RiffMatch::None;
    }

    if !header.starts_with(b"RIFF") {
        return RiffMatch::None;
    }

    if header.len() < 12 {
        return RiffMatch::Truncated("wav".to_string());
    }

    let form_type = &header[8..12];
    let mut evidence = vec![EvidenceItem::new("sig.riff.header")];

    // 1. WAV audio (form_type == b"WAVE")
    if form_type == b"WAVE" {
        evidence.push(EvidenceItem::new("sig.wav.wave_header"));
        let scan_limit = header.len().min(512);
        let mut has_fmt = false;

        if scan_limit >= 16 {
            let mut offset = 12;
            while offset + 8 <= scan_limit {
                let chunk_id = &header[offset..offset + 4];
                let chunk_size = u32::from_le_bytes([
                    header[offset + 4],
                    header[offset + 5],
                    header[offset + 6],
                    header[offset + 7],
                ]) as usize;

                if chunk_id == b"fmt " {
                    has_fmt = true;
                    break;
                }
                // Word-aligned chunk advance
                offset += 8 + ((chunk_size + 1) & !1);
            }
        }

        let verification = if has_fmt {
            evidence.push(EvidenceItem::new("sig.wav.fmt_chunk"));
            Verification::Structure
        } else {
            Verification::Signature
        };

        return RiffMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("wav".to_string()),
            category: "audio".to_string(),
            container_id: Some("riff".to_string()),
            identification: Identification::Format,
            verification,
            inspection: Inspection::Complete,
            recommended_extension: Some("wav".to_string()),
            accepted_extensions: vec!["wav".to_string(), "wave".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // 2. WebP image (form_type == b"WEBP")
    if form_type == b"WEBP" {
        evidence.push(EvidenceItem::new("sig.webp.header"));
        let has_vp8 = header.len() >= 16
            && (&header[12..16] == b"VP8 "
                || &header[12..16] == b"VP8L"
                || &header[12..16] == b"VP8X");
        let verification = if has_vp8 {
            evidence.push(EvidenceItem::new("sig.webp.vp8"));
            Verification::Structure
        } else {
            Verification::Signature
        };

        return RiffMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("webp".to_string()),
            category: "image".to_string(),
            container_id: Some("riff".to_string()),
            identification: Identification::Format,
            verification,
            inspection: Inspection::Complete,
            recommended_extension: Some("webp".to_string()),
            accepted_extensions: vec!["webp".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // 3. AVI video (form_type == b"AVI ")
    if form_type == b"AVI " {
        evidence.push(EvidenceItem::new("sig.avi.header"));
        return RiffMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("avi".to_string()),
            category: "video".to_string(),
            container_id: Some("riff".to_string()),
            identification: Identification::Format,
            verification: Verification::Signature,
            inspection: Inspection::Complete,
            recommended_extension: Some("avi".to_string()),
            accepted_extensions: vec!["avi".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // Unknown RIFF form_type: let downstream specialized rules evaluate
    RiffMatch::None
}
