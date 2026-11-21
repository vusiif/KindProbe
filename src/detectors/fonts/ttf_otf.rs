use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum TtfOtfMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes TrueType (TTF) and OpenType (OTF) font signatures.
pub fn probe_ttf_otf(header: &[u8]) -> TtfOtfMatch {
    if header.len() < 4 {
        if !header.is_empty() && (header.starts_with(&[0x00, 0x01]) || header.starts_with(b"OT")) {
            return TtfOtfMatch::Truncated("ttf".to_string());
        }
        return TtfOtfMatch::None;
    }

    let magic = &header[0..4];
    let is_ttf = magic == [0x00, 0x01, 0x00, 0x00] || magic == b"true";
    let is_otf = magic == b"OTTO";

    if !is_ttf && !is_otf {
        return TtfOtfMatch::None;
    }

    let (fmt_id, magic_evidence_code) = if is_otf {
        ("otf", "sig.font.otf_magic")
    } else {
        ("ttf", "sig.font.ttf_magic")
    };

    let mut evidence = vec![EvidenceItem::new(magic_evidence_code)];

    let verification = if header.len() >= 12 {
        let num_tables = u16::from_be_bytes([header[4], header[5]]);
        if (1..=200).contains(&num_tables) {
            evidence.push(EvidenceItem::new("sig.font.num_tables"));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    TtfOtfMatch::Confirmed(Box::new(DetectionReport {
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
