use crate::detectors::archives::crc32::crc32_ieee;
use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const XZ_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];
pub const ZSTD_MAGIC: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD];

pub enum XzZstdMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes XZ and Zstandard compressed archive formats.
pub fn probe_xz_zstd(header: &[u8]) -> XzZstdMatch {
    // 1. XZ format probe
    if header.len() < 6 {
        if !header.is_empty()
            && (header.starts_with(b"\xFD7z") || header.starts_with(&[0x28, 0xB5]))
        {
            return XzZstdMatch::Truncated(if header.starts_with(b"\xFD7z") {
                "xz".to_string()
            } else {
                "zstd".to_string()
            });
        }
    } else if header.starts_with(XZ_MAGIC) {
        let mut evidence = vec![EvidenceItem::new("sig.xz.magic")];
        let verification = if header.len() >= 12 {
            let flags = &header[6..8];
            let expected_crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            if crc32_ieee(flags) == expected_crc {
                evidence.push(EvidenceItem::new("sig.xz.crc"));
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };

        return XzZstdMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("xz".to_string()),
            category: "archive".to_string(),
            container_id: Some("xz".to_string()),
            identification: Identification::Format,
            verification,
            inspection: Inspection::Complete,
            recommended_extension: Some("xz".to_string()),
            accepted_extensions: vec!["xz".to_string(), "txz".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // 2. Zstandard format probe
    if header.len() >= 4 && header.starts_with(ZSTD_MAGIC) {
        return XzZstdMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("zstd".to_string()),
            category: "archive".to_string(),
            container_id: Some("zstd".to_string()),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("zst".to_string()),
            accepted_extensions: vec!["zst".to_string(), "zstd".to_string(), "tzst".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.zstd.magic")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    XzZstdMatch::None
}
