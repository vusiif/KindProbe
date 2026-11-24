use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

pub enum PngProbeResult {
    Confirmed { has_ihdr: bool },
    Truncated,
    None,
}

pub fn probe_png(header: &[u8]) -> PngProbeResult {
    if header.len() >= 8 && header.starts_with(PNG_SIGNATURE) {
        // Validate IHDR chunk (13 bytes data + 4 bytes 'IHDR')
        if header.len() >= 16 {
            let chunk_len = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
            let chunk_type = &header[12..16];
            if chunk_len == 13 && chunk_type == b"IHDR" {
                return PngProbeResult::Confirmed { has_ihdr: true };
            }
        }
        PngProbeResult::Confirmed { has_ihdr: false }
    } else if !header.is_empty() && header.len() < 8 && PNG_SIGNATURE.starts_with(header) {
        PngProbeResult::Truncated
    } else {
        PngProbeResult::None
    }
}

pub fn make_png_core(has_ihdr: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("png"),
        category: "image",
        container_id: None,
        identification: Identification::Format,
        verification: if has_ihdr {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("png"),
        accepted_extensions: &["png"],
        primary_evidence: Some("sig.png.header"),
        secondary_evidence: if has_ihdr {
            Some("sig.png.ihdr")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_png_report(has_ihdr: bool) -> DetectionReport {
    make_png_core(has_ihdr).to_report(Default::default(), ExtensionStatus::Undetermined)
}
