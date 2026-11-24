use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub const GZIP_MAGIC: &[u8] = &[0x1F, 0x8B];

pub enum GzipMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_gzip_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("gzip"),
        category: "archive",
        container_id: Some("gzip"),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("gz"),
        accepted_extensions: &["gz", "gzip", "tgz"],
        primary_evidence: Some("sig.gzip.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.gzip.header")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_gzip_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 2 || !header.starts_with(GZIP_MAGIC) {
        return None;
    }

    let verification = if header.len() >= 10 {
        let cm = header[2]; // Compression Method: 8 = deflate
        let flg = header[3]; // Flags: bits 5, 6, 7 are reserved and must be 0
        if cm == 8 && (flg & 0xE0) == 0 {
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    Some(make_gzip_core(verification))
}

/// Probes RFC 1952 Gzip compression container format.
pub fn probe_gzip(header: &[u8]) -> GzipMatch {
    if header.len() < 2 {
        if !header.is_empty() && header[0] == 0x1F {
            return GzipMatch::Truncated("gzip".to_string());
        }
        return GzipMatch::None;
    }

    if !header.starts_with(GZIP_MAGIC) {
        return GzipMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.gzip.magic")];
    let verification = if header.len() >= 10 {
        let cm = header[2]; // Compression Method: 8 = deflate
        let flg = header[3]; // Flags: bits 5, 6, 7 are reserved and must be 0
        if cm == 8 && (flg & 0xE0) == 0 {
            evidence.push(EvidenceItem::new("sig.gzip.header"));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    GzipMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("gzip".to_string()),
        category: "archive".to_string(),
        container_id: Some("gzip".to_string()),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("gz".to_string()),
        accepted_extensions: vec!["gz".to_string(), "gzip".to_string(), "tgz".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
