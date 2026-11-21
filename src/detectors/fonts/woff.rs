use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum WoffMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes Web Open Font Format (WOFF 1.0 & WOFF 2.0).
pub fn probe_woff(header: &[u8]) -> WoffMatch {
    if header.len() < 4 {
        if !header.is_empty() && header.starts_with(b"wO") {
            return WoffMatch::Truncated("woff".to_string());
        }
        return WoffMatch::None;
    }

    let magic = &header[0..4];
    let is_woff1 = magic == b"wOFF";
    let is_woff2 = magic == b"wOF2";

    if !is_woff1 && !is_woff2 {
        return WoffMatch::None;
    }

    let (fmt_id, magic_code, header_code) = if is_woff2 {
        ("woff2", "sig.font.woff2_magic", "sig.font.woff2_header")
    } else {
        ("woff", "sig.font.woff_magic", "sig.font.woff_header")
    };

    let mut evidence = vec![EvidenceItem::new(magic_code)];

    let verification = if header.len() >= 14 {
        let num_tables = u16::from_be_bytes([header[12], header[13]]);
        if (1..=200).contains(&num_tables) {
            evidence.push(EvidenceItem::new(header_code));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    WoffMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some(fmt_id.to_string()),
        category: "font".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(fmt_id.to_string()),
        accepted_extensions: vec![fmt_id.to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
