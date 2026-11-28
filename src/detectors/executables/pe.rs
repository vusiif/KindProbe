use super::pe_meta::extract_pe_metadata;
use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub enum PeMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_pe_core(verification: Verification, is_dll: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("pe"),
        category: "executable",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(if is_dll { "dll" } else { "exe" }),
        accepted_extensions: &["exe", "dll", "sys", "efi"],
        primary_evidence: Some("sig.pe.mz_header"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.pe.signature")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

fn valid_pe_header_offset(header: &[u8]) -> Option<usize> {
    if header.len() < 0x40 {
        return None;
    }
    let pe_offset =
        u32::from_le_bytes([header[0x3C], header[0x3D], header[0x3E], header[0x3F]]) as usize;
    let pe_sig_end = pe_offset.checked_add(4)?;
    if pe_offset >= 0x40
        && pe_sig_end <= header.len()
        && &header[pe_offset..pe_sig_end] == b"PE\0\0"
    {
        Some(pe_offset)
    } else {
        None
    }
}

pub fn probe_pe_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 2 || !header.starts_with(b"MZ") {
        return None;
    }

    let (verification, is_dll) = match valid_pe_header_offset(header) {
        Some(pe_offset) => {
            let (dll_flag, _) = extract_pe_metadata(header, pe_offset);
            (Verification::Structure, dll_flag)
        }
        None => (Verification::Signature, false),
    };

    Some(make_pe_core(verification, is_dll))
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
    let (verification, is_dll) = match valid_pe_header_offset(header) {
        Some(pe_offset) => {
            evidence.push(EvidenceItem::new("sig.pe.signature"));
            let (dll_flag, meta_items) = extract_pe_metadata(header, pe_offset);
            evidence.extend(meta_items);
            (Verification::Structure, dll_flag)
        }
        None => (Verification::Signature, false),
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
