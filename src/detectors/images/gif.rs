use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum GifProbeResult {
    Confirmed { has_screen_desc: bool },
    Truncated,
    None,
}

pub fn probe_gif(header: &[u8]) -> GifProbeResult {
    if header.len() >= 6 && (header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a")) {
        let mut has_screen_desc = false;
        if header.len() >= 10 {
            let width = u16::from_le_bytes([header[6], header[7]]);
            let height = u16::from_le_bytes([header[8], header[9]]);
            if width > 0 && height > 0 {
                has_screen_desc = true;
            }
        }
        GifProbeResult::Confirmed { has_screen_desc }
    } else if header.len() >= 2
        && header.len() < 6
        && (b"GIF87a".starts_with(header) || b"GIF89a".starts_with(header))
    {
        GifProbeResult::Truncated
    } else {
        GifProbeResult::None
    }
}

pub fn make_gif_core(has_screen_desc: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("gif"),
        category: "image",
        container_id: None,
        identification: Identification::Format,
        verification: if has_screen_desc {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("gif"),
        accepted_extensions: &["gif"],
        primary_evidence: Some("sig.gif.header"),
        secondary_evidence: if has_screen_desc {
            Some("sig.gif.logical_screen")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_gif_report(has_screen_desc: bool) -> DetectionReport {
    make_gif_core(has_screen_desc).to_report(Default::default(), ExtensionStatus::Undetermined)
}
