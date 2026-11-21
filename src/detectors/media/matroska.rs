use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const EBML_MAGIC: &[u8] = &[0x1A, 0x45, 0xDF, 0xA3];

pub enum MatroskaMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes EBML / Matroska header to detect WebM and MKV formats.
pub fn probe_matroska(header: &[u8]) -> MatroskaMatch {
    if header.len() < 4 {
        if header.len() >= 2 && header.starts_with(&[0x1A, 0x45]) {
            return MatroskaMatch::Truncated("webm".to_string());
        }
        return MatroskaMatch::None;
    }

    if !header.starts_with(EBML_MAGIC) {
        return MatroskaMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.ebml.header")];
    let scan_limit = header.len().min(512);

    // Search for DocType element ID: 0x42, 0x82
    let mut doctype_str = None;
    if scan_limit > 4 {
        for i in 4..scan_limit.saturating_sub(2) {
            if header[i] == 0x42 && header[i + 1] == 0x82 {
                // Next byte is VINT length
                let len_byte = header[i + 2];
                let len = if (len_byte & 0x80) != 0 {
                    (len_byte & 0x7F) as usize
                } else {
                    0
                };
                let val_start = i + 3;
                if len > 0 && val_start + len <= header.len() {
                    if let Ok(s) = std::str::from_utf8(&header[val_start..val_start + len]) {
                        doctype_str = Some(s.trim_matches('\0').to_ascii_lowercase());
                    }
                }
                break;
            }
        }
    }

    if let Some(doctype) = doctype_str {
        if doctype == "webm" {
            evidence.push(EvidenceItem::new("sig.ebml.doctype_webm"));
            return MatroskaMatch::Confirmed(Box::new(DetectionReport {
                schema_version: 1,
                format_id: Some("webm".to_string()),
                category: "video".to_string(),
                container_id: Some("matroska".to_string()),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("webm".to_string()),
                accepted_extensions: vec!["webm".to_string()],
                extension_status: ExtensionStatus::Undetermined,
                evidence,
                warnings: Vec::new(),
                stats: Default::default(),
                candidates: None,
            }));
        } else if doctype == "matroska" {
            evidence.push(EvidenceItem::new("sig.ebml.doctype_mkv"));
            return MatroskaMatch::Confirmed(Box::new(DetectionReport {
                schema_version: 1,
                format_id: Some("mkv".to_string()),
                category: "video".to_string(),
                container_id: Some("matroska".to_string()),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("mkv".to_string()),
                accepted_extensions: vec!["mkv".to_string(), "mka".to_string()],
                extension_status: ExtensionStatus::Undetermined,
                evidence,
                warnings: Vec::new(),
                stats: Default::default(),
                candidates: None,
            }));
        }
    }

    // EBML header present but DocType not found or other
    MatroskaMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("mkv".to_string()),
        category: "video".to_string(),
        container_id: Some("matroska".to_string()),
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("mkv".to_string()),
        accepted_extensions: vec!["mkv".to_string(), "mka".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
