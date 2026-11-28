use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum JpegProbeResult {
    Confirmed { has_segment: bool },
    Truncated,
    None,
}

pub fn probe_jpeg(header: &[u8]) -> JpegProbeResult {
    if header.len() >= 3 && header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF {
        // Validate following marker segment
        let mut has_segment = false;
        if header.len() >= 4 {
            let marker = header[3];
            // Valid JPEG marker after SOI:
            // 0xE0..=0xEF (APP0-APP15), 0xDB (DQT), 0xC0..=0xC3 (SOF0-SOF3), 0xFE (COM)
            if (0xE0..=0xEF).contains(&marker) || marker == 0xDB || (0xC0..=0xC3).contains(&marker) || marker == 0xFE {
                // If we have at least 6 bytes, we can verify segment length
                if header.len() >= 6 {
                    let seg_len = u16::from_be_bytes([header[4], header[5]]) as usize;
                    // Segment length field includes the 2 bytes of the length itself (must be >= 2)
                    if seg_len >= 2 {
                        has_segment = true;
                    }
                } else {
                    has_segment = true;
                }
            }
        }

        JpegProbeResult::Confirmed { has_segment }
    } else if header.len() == 2 && header[0] == 0xFF && header[1] == 0xD8 {
        JpegProbeResult::Truncated
    } else {
        JpegProbeResult::None
    }
}

pub fn make_jpeg_core(has_segment: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("jpeg"),
        category: "image",
        container_id: None,
        identification: Identification::Format,
        verification: if has_segment {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("jpg"),
        accepted_extensions: &["jpg", "jpeg", "jfif", "jpe"],
        primary_evidence: Some("sig.jpeg.soi"),
        secondary_evidence: if has_segment {
            Some("sig.jpeg.segment")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_jpeg_report(has_segment: bool) -> DetectionReport {
    make_jpeg_core(has_segment).to_report(Default::default(), ExtensionStatus::Undetermined)
}
