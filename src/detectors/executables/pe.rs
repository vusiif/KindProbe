use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};
use super::pe_meta::extract_pe_metadata;

pub enum PeMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes Windows Portable Executable (PE) / DOS MZ format.
pub fn probe_pe(header: &[u8]) -> PeMatch {
    if header.len() < 2 {
        if !header.is_empty() && header[0] == b'M' {
            return PeMatch::Truncated("pe".to_string());
        }
        return PeMatch::None;
    }

    if !header.starts_with(b"MZ") {
        return PeMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.pe.mz_header")];
    let (verification, is_dll) = if header.len() >= 0x40 {
        let pe_offset = u32::from_le_bytes([header[0x3C], header[0x3D], header[0x3E], header[0x3F]]) as usize;
        if pe_offset >= 0x40 && pe_offset + 4 <= header.len() && &header[pe_offset..pe_offset + 4] == b"PE\0\0" {
            evidence.push(EvidenceItem::new("sig.pe.signature"));
            let (dll_flag, meta_items) = extract_pe_metadata(header, pe_offset);
            evidence.extend(meta_items);
            (Verification::Structure, dll_flag)
        } else {
            (Verification::Signature, false)
        }
    } else {
        (Verification::Signature, false)
    };

    let rec_ext = if is_dll { "dll" } else { "exe" };

    PeMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("pe".to_string()),
        category: "executable".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec_ext.to_string()),
        accepted_extensions: vec![
            "exe".to_string(),
            "dll".to_string(),
            "sys".to_string(),
            "efi".to_string(),
        ],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
