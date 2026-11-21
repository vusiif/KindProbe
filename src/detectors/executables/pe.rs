use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

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

    // Need offset 0x3C (60) to read e_lfanew
    if header.len() < 0x40 {
        return PeMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("pe".to_string()),
            category: "executable".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Signature,
            inspection: Inspection::Complete,
            recommended_extension: Some("exe".to_string()),
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
        }));
    }

    let e_lfanew = u32::from_le_bytes([header[0x3C], header[0x3D], header[0x3E], header[0x3F]]) as usize;

    // PE signature check
    if e_lfanew >= 0x40 && e_lfanew + 4 <= header.len() && &header[e_lfanew..e_lfanew + 4] == b"PE\0\0" {
        evidence.push(EvidenceItem::new("sig.pe.signature"));

        // Check COFF characteristics at e_lfanew + 22
        let mut is_dll = false;
        if e_lfanew + 24 <= header.len() {
            let characteristics = u16::from_le_bytes([
                header[e_lfanew + 22],
                header[e_lfanew + 23],
            ]);
            is_dll = (characteristics & 0x2000) != 0; // IMAGE_FILE_DLL
        }

        let rec_ext = if is_dll { "dll" } else { "exe" };

        return PeMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("pe".to_string()),
            category: "executable".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
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
        }));
    }

    // MZ header verified, but PE header not reached or absent (DOS MZ executable)
    PeMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("pe".to_string()),
        category: "executable".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("exe".to_string()),
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
