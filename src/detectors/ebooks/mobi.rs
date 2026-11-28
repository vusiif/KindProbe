use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum MobiMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_mobi_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("mobi"),
        category: "document",
        container_id: Some("palmdoc"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("mobi"),
        accepted_extensions: &["mobi", "azw", "prc"],
        primary_evidence: Some("sig.mobi.header"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_palmdoc_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("mobi"),
        category: "document",
        container_id: Some("palmdoc"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("prc"),
        accepted_extensions: &["mobi", "prc", "pdb"],
        primary_evidence: Some("sig.palmdoc.header"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_mobi_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 68 {
        return None;
    }

    let type_creator = &header[60..68];
    if type_creator == b"BOOKMOBI" {
        return Some(make_mobi_core());
    }

    if type_creator == b"BOOKTEXt" {
        return Some(make_palmdoc_core());
    }

    None
}

/// Probes Mobipocket / Kindle / PalmDOC e-book signatures.
pub fn probe_mobi(header: &[u8]) -> MobiMatch {
    if header.len() < 68 {
        if header.len() >= 64 && &header[60..64] == b"BOOK" {
            return MobiMatch::Truncated("mobi".to_string());
        }
        return MobiMatch::None;
    }

    match probe_mobi_core(header) {
        Some(core) => MobiMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => MobiMatch::None,
    }
}
