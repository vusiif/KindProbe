use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum MobiMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes Mobipocket / Kindle / PalmDOC e-book signatures.
pub fn probe_mobi(header: &[u8]) -> MobiMatch {
    if header.len() < 68 {
        if header.len() >= 64 && &header[60..64] == b"BOOK" {
            return MobiMatch::Truncated("mobi".to_string());
        }
        return MobiMatch::None;
    }

    let type_creator = &header[60..68];
    if type_creator == b"BOOKMOBI" {
        return MobiMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("mobi".to_string()),
            category: "document".to_string(),
            container_id: Some("palmdoc".to_string()),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("mobi".to_string()),
            accepted_extensions: vec!["mobi".to_string(), "azw".to_string(), "prc".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.mobi.header")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    if type_creator == b"BOOKTEXt" {
        return MobiMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("mobi".to_string()),
            category: "document".to_string(),
            container_id: Some("palmdoc".to_string()),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("prc".to_string()),
            accepted_extensions: vec!["mobi".to_string(), "prc".to_string(), "pdb".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.palmdoc.header")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    MobiMatch::None
}
