use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum MachoMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes Mach-O binary format (macOS / iOS / Darwin) and Java class disambiguation.
pub fn probe_macho(header: &[u8]) -> MachoMatch {
    if header.len() < 4 {
        if !header.is_empty() && (header.starts_with(&[0xFE, 0xED]) || header.starts_with(&[0xCF, 0xFA])) {
            return MachoMatch::Truncated("macho".to_string());
        }
        return MachoMatch::None;
    }

    let magic = &header[..4];

    // Mach-O Single Architecture (32-bit & 64-bit, BE & LE)
    let is_macho_single = magic == [0xFE, 0xED, 0xFA, 0xCE]
        || magic == [0xCE, 0xFA, 0xED, 0xFE]
        || magic == [0xFE, 0xED, 0xFA, 0xCF]
        || magic == [0xCF, 0xFA, 0xED, 0xFE];

    if is_macho_single {
        return MachoMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("macho".to_string()),
            category: "executable".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("dylib".to_string()),
            accepted_extensions: vec![
                "dylib".to_string(),
                "bin".to_string(),
                "bundle".to_string(),
            ],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.macho.magic")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // Mach-O FAT / Universal Binary (0xCAFEBABE or 0xBEBAFECA) vs Java .class
    if magic == [0xCA, 0xFE, 0xBA, 0xBE] {
        if header.len() >= 8 {
            let minor = u16::from_be_bytes([header[4], header[5]]);
            let major = u16::from_be_bytes([header[6], header[7]]);

            // Java class files typically have minor == 0 and major >= 43 (JDK 1.1+)
            if minor == 0 && (43..=100).contains(&major) {
                return MachoMatch::Confirmed(Box::new(DetectionReport {
                    schema_version: 1,
                    format_id: Some("class".to_string()),
                    category: "executable".to_string(),
                    container_id: None,
                    identification: Identification::Format,
                    verification: Verification::Structure,
                    inspection: Inspection::Complete,
                    recommended_extension: Some("class".to_string()),
                    accepted_extensions: vec!["class".to_string()],
                    extension_status: ExtensionStatus::Undetermined,
                    evidence: vec![EvidenceItem::new("sig.java.class")],
                    warnings: Vec::new(),
                    stats: Default::default(),
                    candidates: None,
                }));
            }
        }

        // Otherwise Mach-O Universal Binary
        return MachoMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("macho".to_string()),
            category: "executable".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("dylib".to_string()),
            accepted_extensions: vec![
                "dylib".to_string(),
                "bin".to_string(),
                "bundle".to_string(),
            ],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.macho.fat_magic")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    if magic == [0xBE, 0xBA, 0xFE, 0xCA] {
        return MachoMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("macho".to_string()),
            category: "executable".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("dylib".to_string()),
            accepted_extensions: vec![
                "dylib".to_string(),
                "bin".to_string(),
                "bundle".to_string(),
            ],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.macho.fat_magic")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    MachoMatch::None
}
