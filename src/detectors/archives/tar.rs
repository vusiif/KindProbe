use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub enum TarMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_tar_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("tar"),
        category: "archive",
        container_id: Some("tar"),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("tar"),
        accepted_extensions: &["tar"],
        primary_evidence: Some("sig.tar.ustar"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.tar.checksum")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_tar_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 512 {
        return None;
    }
    let verification = check_tar_header(header)?;
    Some(make_tar_core(verification))
}

/// Checks if a 512-byte block is a valid POSIX ustar tar header.
pub fn check_tar_header(header: &[u8]) -> Option<Verification> {
    if header.len() < 512 {
        return None;
    }
    let magic = &header[257..263];
    if magic != b"ustar\0" && magic != b"ustar " {
        return None;
    }

    let chk_str = std::str::from_utf8(&header[148..156])
        .unwrap_or("")
        .trim()
        .trim_matches('\0')
        .trim();
    let expected_chk = u32::from_str_radix(chk_str, 8).unwrap_or(u32::MAX);

    let mut sum: u32 = 0;
    for (i, &b) in header[..512].iter().enumerate() {
        sum += if (148..156).contains(&i) {
            0x20
        } else {
            b as u32
        };
    }

    if sum == expected_chk {
        Some(Verification::Structure)
    } else {
        Some(Verification::Signature)
    }
}

/// Probes POSIX tar (ustar) archive format.
pub fn probe_tar(header: &[u8]) -> TarMatch {
    if header.len() < 512 {
        if header.len() >= 265
            && (&header[257..262] == b"ustar" || &header[257..264] == b"ustar\0\0")
        {
            return TarMatch::Truncated("tar".to_string());
        }
        return TarMatch::None;
    }

    let verification = match check_tar_header(header) {
        Some(v) => v,
        None => return TarMatch::None,
    };

    let mut evidence = vec![EvidenceItem::new("sig.tar.ustar")];
    if verification == Verification::Structure {
        evidence.push(EvidenceItem::new("sig.tar.checksum"));
    }

    TarMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("tar".to_string()),
        category: "archive".to_string(),
        container_id: Some("tar".to_string()),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("tar".to_string()),
        accepted_extensions: vec!["tar".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
