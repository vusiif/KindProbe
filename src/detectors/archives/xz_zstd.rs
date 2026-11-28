use crate::detectors::archives::crc32::crc32_ieee;
use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub const XZ_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];
pub const ZSTD_MAGIC: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD];

pub enum XzZstdMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_xz_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("xz"),
        category: "archive",
        container_id: Some("xz"),
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("xz"),
        accepted_extensions: &["xz", "txz"],
        primary_evidence: Some("sig.xz.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.xz.crc")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_zstd_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("zstd"),
        category: "archive",
        container_id: Some("zstd"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("zst"),
        accepted_extensions: &["zst", "zstd", "tzst"],
        primary_evidence: Some("sig.zstd.magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_xz_zstd_core(header: &[u8]) -> Option<DetectionCore> {
    if header.starts_with(XZ_MAGIC) {
        let verification = if header.len() >= 12 {
            let flags = &header[6..8];
            let expected_crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            if crc32_ieee(flags) == expected_crc {
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };
        return Some(make_xz_core(verification));
    }

    if header.len() >= 4 && header.starts_with(ZSTD_MAGIC) {
        return Some(make_zstd_core());
    }

    None
}

/// Probes XZ and Zstandard compressed archive formats.
pub fn probe_xz_zstd(header: &[u8]) -> XzZstdMatch {
    if let Some(core) = probe_xz_zstd_core(header) {
        return XzZstdMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        ));
    }

    // Truncated XZ / Zstd prefixes
    if header.len() >= 2 && header.len() < 6 && XZ_MAGIC.starts_with(header) {
        return XzZstdMatch::Truncated("xz".to_string());
    }
    if header.len() >= 2 && header.len() < 4 && ZSTD_MAGIC.starts_with(header) {
        return XzZstdMatch::Truncated("zstd".to_string());
    }

    XzZstdMatch::None
}
