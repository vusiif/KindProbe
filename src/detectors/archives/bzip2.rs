use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub const BZIP2_MAGIC: &[u8] = b"BZh";

pub enum Bzip2Match {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_bzip2_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("bzip2"),
        category: "archive",
        container_id: Some("bzip2"),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("bz2"),
        accepted_extensions: &["bz2", "bzip2", "tbz2", "tbz"],
        primary_evidence: Some("sig.bzip2.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.bzip2.header")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_bzip2_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 3 || !header.starts_with(BZIP2_MAGIC) {
        return None;
    }

    let verification = if header.len() >= 10 {
        let block_size = header[3];
        let block_magic = &header[4..10];
        let valid_block = block_magic == [0x31, 0x41, 0x59, 0x26, 0x53, 0x59]
            || block_magic == [0x17, 0x72, 0x45, 0x38, 0x50, 0x90];

        if (b'1'..=b'9').contains(&block_size) && valid_block {
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    Some(make_bzip2_core(verification))
}

/// Probes Bzip2 compressed archive format.
pub fn probe_bzip2(header: &[u8]) -> Bzip2Match {
    if header.len() < 3 {
        if !header.is_empty() && header.starts_with(b"BZ") {
            return Bzip2Match::Truncated("bzip2".to_string());
        }
        return Bzip2Match::None;
    }

    if !header.starts_with(BZIP2_MAGIC) {
        return Bzip2Match::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.bzip2.magic")];
    let verification = if header.len() >= 10 {
        let block_size = header[3];
        // Block magic: Pi constant "1AY&SY" (0x314159265359) or EOS marker (0x177245385090)
        let block_magic = &header[4..10];
        let valid_block = block_magic == [0x31, 0x41, 0x59, 0x26, 0x53, 0x59]
            || block_magic == [0x17, 0x72, 0x45, 0x38, 0x50, 0x90];

        if (b'1'..=b'9').contains(&block_size) && valid_block {
            evidence.push(EvidenceItem::new("sig.bzip2.header"));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    Bzip2Match::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("bzip2".to_string()),
        category: "archive".to_string(),
        container_id: Some("bzip2".to_string()),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("bz2".to_string()),
        accepted_extensions: vec![
            "bz2".to_string(),
            "bzip2".to_string(),
            "tbz2".to_string(),
            "tbz".to_string(),
        ],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
