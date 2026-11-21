use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum TarMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
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

    // Check ustar magic at offset 257 (6 bytes)
    let magic = &header[257..263];
    let is_ustar = magic == b"ustar\0" || magic == b"ustar ";

    if !is_ustar {
        return TarMatch::None;
    }

    // Parse checksum at offset 148..156 (8 bytes octal ASCII string)
    let chk_slice = &header[148..156];
    let chk_str = std::str::from_utf8(chk_slice)
        .unwrap_or("")
        .trim()
        .trim_matches('\0')
        .trim();
    let expected_chk = u32::from_str_radix(chk_str, 8).unwrap_or(u32::MAX);

    // Compute checksum treating the 8 checksum bytes as spaces (0x20)
    let mut sum: u32 = 0;
    for (i, &b) in header[..512].iter().enumerate() {
        if (148..156).contains(&i) {
            sum += 0x20;
        } else {
            sum += b as u32;
        }
    }

    let mut evidence = vec![EvidenceItem::new("sig.tar.ustar")];
    let verification = if sum == expected_chk {
        evidence.push(EvidenceItem::new("sig.tar.checksum"));
        Verification::Structure
    } else {
        Verification::Signature
    };

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
