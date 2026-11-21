use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const ELF_MAGIC: &[u8] = &[0x7F, 0x45, 0x4C, 0x46];

pub enum ElfMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes ELF header (Executable and Linkable Format).
pub fn probe_elf(header: &[u8]) -> ElfMatch {
    if header.len() < 4 {
        if !header.is_empty() && header.starts_with(&[0x7F, 0x45]) {
            return ElfMatch::Truncated("elf".to_string());
        }
        return ElfMatch::None;
    }

    if !header.starts_with(ELF_MAGIC) {
        return ElfMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.elf.magic")];

    // ELF header requires at least 16 bytes for e_ident
    if header.len() < 16 {
        return ElfMatch::Truncated("elf".to_string());
    }

    let class_byte = header[4]; // 1 = 32-bit, 2 = 64-bit
    let data_byte = header[5];  // 1 = Little-endian, 2 = Big-endian
    let version_byte = header[6]; // 1 = EV_CURRENT

    let valid_ident = (class_byte == 1 || class_byte == 2)
        && (data_byte == 1 || data_byte == 2)
        && version_byte == 1;

    let verification = if valid_ident {
        evidence.push(EvidenceItem::new("sig.elf.header"));
        Verification::Structure
    } else {
        Verification::Signature
    };

    ElfMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("elf".to_string()),
        category: "executable".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("elf".to_string()),
        accepted_extensions: vec![
            "elf".to_string(),
            "so".to_string(),
            "o".to_string(),
            "bin".to_string(),
        ],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
