use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiffMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum RiffMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_wav_core(has_fmt: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("wav"),
        category: "audio",
        container_id: Some("riff"),
        identification: Identification::Format,
        verification: if has_fmt {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("wav"),
        accepted_extensions: &["wav", "wave"],
        primary_evidence: Some("sig.riff.header"),
        secondary_evidence: if has_fmt {
            Some("sig.wav.fmt_chunk")
        } else {
            Some("sig.wav.wave_header")
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_webp_core(has_vp8: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("webp"),
        category: "image",
        container_id: Some("riff"),
        identification: Identification::Format,
        verification: if has_vp8 {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("webp"),
        accepted_extensions: &["webp"],
        primary_evidence: Some("sig.riff.header"),
        secondary_evidence: if has_vp8 {
            Some("sig.webp.vp8")
        } else {
            Some("sig.webp.header")
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_avi_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("avi"),
        category: "video",
        container_id: Some("riff"),
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("avi"),
        accepted_extensions: &["avi"],
        primary_evidence: Some("sig.riff.header"),
        secondary_evidence: Some("sig.avi.header"),
        warning: None,
        candidate: None,
    }
}

fn has_wave_fmt_chunk(header: &[u8]) -> bool {
    let scan_limit = header.len().min(512);
    if scan_limit < 16 {
        return false;
    }

    let mut offset = 12usize;
    while offset + 8 <= scan_limit {
        let chunk_id = &header[offset..offset + 4];
        if chunk_id == b"fmt " {
            return true;
        }
        let chunk_size = u32::from_le_bytes([
            header[offset + 4],
            header[offset + 5],
            header[offset + 6],
            header[offset + 7],
        ]) as usize;
        if chunk_size > scan_limit {
            break;
        }
        let Some(padded_size) = chunk_size.checked_add(1).map(|v| v & !1) else {
            break;
        };
        let Some(next_offset) = offset
            .checked_add(8)
            .and_then(|v| v.checked_add(padded_size))
        else {
            break;
        };
        offset = next_offset;
    }

    false
}

enum RiffSubtype {
    Wav { has_fmt: bool },
    Webp { has_vp8: bool },
    Avi,
}

fn parse_riff_subtype(header: &[u8]) -> Option<RiffSubtype> {
    let form_type = &header[8..12];
    if form_type == b"WAVE" {
        return Some(RiffSubtype::Wav {
            has_fmt: has_wave_fmt_chunk(header),
        });
    }
    if form_type == b"WEBP" {
        let has_vp8 = header.len() >= 16
            && (&header[12..16] == b"VP8 "
                || &header[12..16] == b"VP8L"
                || &header[12..16] == b"VP8X");
        return Some(RiffSubtype::Webp { has_vp8 });
    }
    if form_type == b"AVI " {
        return Some(RiffSubtype::Avi);
    }
    None
}

pub fn probe_riff_core(header: &[u8]) -> RiffMatchCore {
    if header.len() < 4 {
        if header.len() >= 2 && header.starts_with(b"RI") {
            return RiffMatchCore::Truncated("wav");
        }
        return RiffMatchCore::None;
    }

    if !header.starts_with(b"RIFF") {
        return RiffMatchCore::None;
    }

    if header.len() < 12 {
        return RiffMatchCore::Truncated("wav");
    }

    match parse_riff_subtype(header) {
        Some(RiffSubtype::Wav { has_fmt }) => RiffMatchCore::Confirmed(make_wav_core(has_fmt)),
        Some(RiffSubtype::Webp { has_vp8 }) => RiffMatchCore::Confirmed(make_webp_core(has_vp8)),
        Some(RiffSubtype::Avi) => RiffMatchCore::Confirmed(make_avi_core()),
        None => RiffMatchCore::None,
    }
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

    let mut evidence = vec![EvidenceItem::new("sig.riff.header")];

    match parse_riff_subtype(header) {
        Some(RiffSubtype::Wav { has_fmt }) => {
            evidence.push(EvidenceItem::new("sig.wav.wave_header"));
            let verification = if has_fmt {
                evidence.push(EvidenceItem::new("sig.wav.fmt_chunk"));
                Verification::Structure
            } else {
                Verification::Signature
            };

            RiffMatch::Confirmed(Box::new(DetectionReport {
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
            }))
        }
        Some(RiffSubtype::Webp { has_vp8 }) => {
            evidence.push(EvidenceItem::new("sig.webp.header"));
            let verification = if has_vp8 {
                evidence.push(EvidenceItem::new("sig.webp.vp8"));
                Verification::Structure
            } else {
                Verification::Signature
            };

            RiffMatch::Confirmed(Box::new(DetectionReport {
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
            }))
        }
        Some(RiffSubtype::Avi) => {
            evidence.push(EvidenceItem::new("sig.avi.header"));
            RiffMatch::Confirmed(Box::new(DetectionReport {
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
            }))
        }
        None => {
            // Unknown RIFF form_type: let downstream specialized rules evaluate
            RiffMatch::None
        }
    }
}
